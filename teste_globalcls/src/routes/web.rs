use rustpress_web::{App, Request, Response};

pub fn register(app: &mut App) -> Result<(), rustpress_web::HandlerError> {
    app.get("/", |_request: Request, response: Response| {
        Ok(response.html("<h1>RustPress está a funcionar!</h1>"))
    })?;

    Ok(())
}
