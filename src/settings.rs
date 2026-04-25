use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub refresh_interval_secs: f64,
    pub port_range_min: u16,
    pub port_range_max: u16,
    pub show_system_ports: bool,
    pub show_all_users: bool,
    pub appearance: Appearance,
    pub launch_at_login: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            refresh_interval_secs: 3.0,
            port_range_min: 1024,
            port_range_max: 65535,
            show_system_ports: false,
            show_all_users: false,
            appearance: Appearance::System,
            launch_at_login: false,
        }
    }
}

impl Settings {
    pub fn normalized(mut self) -> Self {
        if self.port_range_min > self.port_range_max {
            std::mem::swap(&mut self.port_range_min, &mut self.port_range_max);
        }
        self.refresh_interval_secs = self.refresh_interval_secs.clamp(1.0, 30.0);
        self
    }

    pub fn load() -> anyhow::Result<Self> {
        let s: Settings = confy::load("port-monitor", None)?;
        Ok(s.normalized())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        confy::store("port-monitor", None, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let s = Settings::default();
        assert_eq!(s.refresh_interval_secs, 3.0);
        assert_eq!(s.port_range_min, 1024);
        assert_eq!(s.port_range_max, 65535);
        assert!(!s.show_system_ports);
        assert!(!s.show_all_users);
        assert!(!s.launch_at_login);
        assert_eq!(s.appearance, Appearance::System);
    }

    #[test]
    fn normalized_swaps_inverted_range() {
        let s = Settings {
            port_range_min: 9000,
            port_range_max: 1000,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(s.port_range_min, 1000);
        assert_eq!(s.port_range_max, 9000);
    }

    #[test]
    fn normalized_clamps_interval() {
        let s = Settings {
            refresh_interval_secs: 0.01,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(s.refresh_interval_secs, 1.0);

        let s = Settings {
            refresh_interval_secs: 999.0,
            ..Settings::default()
        }
        .normalized();
        assert_eq!(s.refresh_interval_secs, 30.0);
    }

    #[test]
    fn serde_roundtrip() {
        let original = Settings::default();
        let serialized = toml::to_string(&original).unwrap();
        let parsed: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.port_range_min, original.port_range_min);
        assert_eq!(parsed.appearance, original.appearance);
    }
}
