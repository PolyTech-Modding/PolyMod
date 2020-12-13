use actix_web::{web, HttpResponse, HttpRequest};

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadJsonData {
    test: String,
}

pub async fn upload(req: HttpRequest, data: web::Json<UploadJsonData>) -> HttpResponse {
    println!("AAAAAAAA");
    format!("{:#?}", data).into()
}
