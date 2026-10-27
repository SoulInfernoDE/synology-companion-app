//! An optional task in DSM's Task Scheduler that runs at boot and applies every
//! recipe again that the journal says was applied but whose check now fails,
//! as after a DSM update, without the app.
//!
//! The script it runs is written by the app, owned by root and readable only by
//! root; it checks each recipe script against its SHA-256 before running it and
//! adds a line to the journal for every recipe it applies.

use companion_ssh::{Connection, quote};

use crate::{Entry, Error, Place, Recipe, last_applied, place};

/// Name of the task in DSM's Task Scheduler.
pub const TASK: &str = "Syno Companion: re-apply recipes";

fn json_string(text: &str) -> String {
    serde_json::Value::String(text.to_owned()).to_string()
}

/// The boot script for the recipes the journal says are applied.
pub(crate) fn script(recipes: &[Recipe], entries: &[Entry], place: &Place) -> String {
    let journal = quote(&format!("{}/journal.jsonl", place.root()));
    let mut text = String::from(
        "#!/bin/sh\n# Written by Syno Companion: applies recipes again that a DSM update removed.\n\
         . /etc.defaults/VERSION\ndsm=\"$productversion-$buildnumber\"\n",
    );
    for recipe in recipes {
        let Some(entry) = last_applied(entries, &recipe.id) else {
            continue;
        };
        let Ok(flags) = recipe.flags(&entry.options) else {
            continue;
        };
        let dir = place.scripts(recipe);
        let script = format!("{dir}/{}", recipe.file);
        let options = serde_json::to_string(&entry.options).unwrap_or_else(|_| "[]".into());
        text.push_str(&format!(
            r#"
# {id} {version}
if ! ( {check} ) >/dev/null 2>&1; then
  if ( cd {dir} && echo "{sha}  {file}" | sha256sum -c --status ); then
    ( cd {dir} && {apply} ) </dev/null >/dev/null 2>&1 && ok=true || ok=false
    printf '{{"time":%s,"recipe":%s,"version":%s,"action":"apply","options":%s,"ok":%s,"dsm":"%s","by":"boot task"}}\n' "$(date +%s)" {id_json} {version_json} {options_json} "$ok" "$dsm" >> {journal}
  fi
fi
"#,
            id = recipe.id,
            version = recipe.version,
            check = recipe.steps.check,
            dir = quote(&dir),
            sha = recipe.sha256,
            file = quote(&recipe.file),
            apply = recipe.command(&recipe.steps.apply, &script, &flags),
            id_json = quote(&json_string(&recipe.id)),
            version_json = quote(&json_string(&recipe.version)),
            options_json = quote(&options),
        ));
    }
    text
}

fn webapi(method: &str, extra: &str) -> String {
    format!(
        "synowebapi --exec api=SYNO.Core.EventScheduler version=1 method={method} task_name={name} {extra}",
        name = quote(&json_string(TASK)),
    )
}

/// Whether the boot task is set up, judged by its folder, which needs no root.
pub async fn installed(connection: &Connection) -> Result<bool, Error> {
    let output = connection
        .run(r#"test -d "$HOME/.syno-companion/boot""#)
        .await?;
    Ok(output.success())
}

/// Writes the boot script and creates (or replaces) the boot task. Returns the
/// number of recipes the script covers.
pub async fn install(
    connection: &Connection,
    recipes: &[Recipe],
    entries: &[Entry],
    password: &str,
) -> Result<usize, Error> {
    let place = place(connection).await?;
    let covered = recipes
        .iter()
        .filter(|recipe| last_applied(entries, &recipe.id).is_some())
        .count();
    let text = script(recipes, entries, &place);
    let path = format!("{}/boot/reapply.sh", place.root());
    let operation = json_string(&format!("/bin/sh {}", quote(&path)));
    let create = webapi(
        "create",
        &format!(
            "owner='{{\"0\":\"root\"}}' event='\"bootup\"' enable=true depend_on_task='\"\"' \
             notify_enable=false notify_mail='\"\"' notify_if_error=false \
             operation_type='\"script\"' operation={operation}",
            operation = quote(&operation),
        ),
    );
    // The script comes on standard input after the password line; sudo reads
    // only that line, so the shell below gets the rest.
    let command = format!(
        r#"dir={dir}
mkdir -p "$dir" && chown root:root "$dir" && chmod 700 "$dir" || exit 1
cat > "$dir/reapply.sh" && chown root:root "$dir/reapply.sh" && chmod 700 "$dir/reapply.sh" || exit 1
{delete} >/dev/null 2>&1
{create}"#,
        dir = quote(&format!("{}/boot", place.root())),
        delete = webapi("delete", ""),
        create = create,
    );
    let output = connection
        .run_sudo_with_input(&command, password, text.as_bytes())
        .await?;
    if output.sudo_refused() {
        return Err(Error::Sudo);
    }
    if !output.success() || output.stdout.contains("\"success\" : false") {
        return Err(Error::Failed(format!("{}{}", output.stdout, output.stderr)));
    }
    Ok(covered)
}

/// Deletes the boot task and its script.
pub async fn remove(connection: &Connection, password: &str) -> Result<(), Error> {
    let place = place(connection).await?;
    let command = format!(
        "{delete} >/dev/null 2>&1; rm -rf {dir}",
        delete = webapi("delete", ""),
        dir = quote(&format!("{}/boot", place.root())),
    );
    let output = connection.run_sudo(&command, password).await?;
    if output.sudo_refused() {
        return Err(Error::Sudo);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin;

    #[test]
    fn the_script_covers_applied_recipes_only() {
        let recipes = builtin();
        let place = Place {
            home: "/var/services/homes/admin".into(),
        };
        let entry = Entry {
            time: 1,
            recipe: "third-party-drives".into(),
            version: "v3.6.139".into(),
            action: "apply".into(),
            options: vec!["noupdate".into()],
            ok: true,
            dsm: "7.2.2-72806".into(),
            backup: None,
            by: "x".into(),
        };
        assert!(!script(&recipes, &[], &place).contains("third-party-drives v3.6.139"));
        let text = script(&recipes, &[entry], &place);
        assert!(text.contains("# third-party-drives v3.6.139"));
        assert!(text.contains("bash '/var/services/homes/admin/.syno-companion/recipes/third-party-drives/v3.6.139/syno_hdd_db.sh' -e '--noupdate'"));
        assert!(text.contains("sha256sum -c --status"));
        assert!(text.contains(r#"'"third-party-drives"'"#));
        assert!(text.contains(r#"'["noupdate"]'"#));
    }

    #[test]
    fn web_api_calls_quote_json_strings() {
        assert_eq!(
            webapi("get", ""),
            r#"synowebapi --exec api=SYNO.Core.EventScheduler version=1 method=get task_name='"Syno Companion: re-apply recipes"' "#
        );
    }
}
