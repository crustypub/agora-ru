use actix_web::{get, web, HttpRequest, HttpResponse, Error};
use tokio::sync::mpsc;
use crate::helpers::api::AuthenticatedUser;
use crate::models::app::AppState;
use crate::models::chat::WsMessage;
use crate::helpers::chats::ws_session_loop;

#[get("/ws/chat")]
pub async fn chat_ws_route(
    user: AuthenticatedUser,
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user_id = user.id;

    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let (tx, rx) = mpsc::unbounded_channel::<WsMessage>();

    app_state.chat_server.sessions
        .entry(user_id)
        .or_insert_with(Vec::new)
        .push(tx);

    // Используем actix_web::rt::spawn, так как ws_session_loop внутри себя
    // содержит не-Send тип actix_ws::MessageStream.
    actix_web::rt::spawn(ws_session_loop(
        app_state.get_ref().chat_server.clone(),
        app_state.get_ref().pool.clone(),
        app_state.get_ref().s3_public_client.clone(),
        user_id,
        session,
        msg_stream,
        rx,
    ));

    Ok(res)
}
