use udev;

/// Returns a vector of all `ID_INPUT` properties on this device
pub fn input_id_udev_props(d: &udev::Device) -> Vec<String> {
    let excluded = vec!["ID_INPUT_HEIGHT_MM", "ID_INPUT_WIDTH"];
    let mut caps = Vec::new();

    for property in d.properties() {
        if let Some(name) = property.name().to_str() {
            let namestr = String::from(name);
            if namestr.starts_with("ID_INPUT") {
                if excluded.iter().any(|e| name == *e) {
                    continue;
                }
                if let Some(v) = property.value().to_str() {
                    if v != "0" {
                        caps.push(String::from(name));
                    }
                }
            }
        }
    }
    caps
}
