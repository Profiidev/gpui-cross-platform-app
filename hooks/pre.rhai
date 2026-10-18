// Derived names used across Android, iOS and desktop.
let name = variable::get("project-name");

// iOS bundle IDs allow only [A-Za-z0-9.-], Android segments no "-"
let id_segment = name;
id_segment.replace("-", "");
id_segment.replace("_", "");
variable::set("app_id", variable::get("app_id_prefix") + "." + id_segment.to_lower());

// PascalCase display / Xcode target name, e.g. my-app -> MyApp
let app_name = "";
for part in name.split("-") {
  for sub in part.split("_") {
    if sub.len() > 0 {
      app_name += sub.sub_string(0, 1).to_upper() + sub.sub_string(1);
    }
  }
}
variable::set("app_name", app_name);

// Inert in the template repo (GitHub only runs *.yml), active in generated repos
file::rename(".github/workflows/renovate.yml.tmpl", ".github/workflows/renovate.yml");
