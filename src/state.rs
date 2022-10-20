use futures::channel::mpsc::UnboundedSender;


pub struct AppState {
    // pub sender: UnboundedSender<String>,
    pub wechat_robot: Option<String>
}
