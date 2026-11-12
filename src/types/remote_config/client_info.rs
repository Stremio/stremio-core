#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ClientInfo {
    pub platform: String,
    pub os: Option<String>,
    pub app_version: Option<String>,
    pub shell_version: Option<String>,
    pub installation_id: Option<String>,
    pub notification_templates: Vec<String>,
}
