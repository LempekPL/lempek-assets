use actix_web::{HttpResponse, Responder, Scope, web, get};
use crate::DOMAIN;

pub fn frontend() -> Scope {
    web::scope("/front")
        .service(upload)
}

#[get("/upload")]
async fn upload() -> impl Responder {
    let paths = std::fs::read_dir("./static/files").unwrap();
    let mut data = "".to_string();
    for path in paths {
        let a = path.unwrap().path().display().to_string().replace("./static/files\\", &format!("{}/files/", DOMAIN));
        data += &format!("<a href=\"{}\" target=\"_blank\">{}</a><br/>", &a, a);
    }
    let html = r#"<html>
        <head><title>Upload</title></head>
        <body>
<input type="file" multiple name="file" id="files"/>
<input type="password" name="code" id="code"/>
replace?:
<input type="checkbox" name="replace" id="replace"/>
<button type="submit" onclick="click()" id="but">Submit</button>
<div id="bottom"></div>
<br />
<br />
<br />
<br />
<div>"#.to_string() + &data + r#"</div>

<script>
const bottom = document.getElementById('bottom');
const fileInput = document.getElementById('files');
const code = document.getElementById('code');
const replace = document.getElementById('replace');
but.onclick = async () => {
        const formData = new FormData();
        for (let file of fileInput.files) {
            formData.append('file', file);
        }
        formData.append('code', code.value);
        formData.append('replace', replace.checked);
        let res = await fetch("/api/v1/upload", {
            method: 'POST',
              body: formData
        });
        let json = await res.json();
        bottom.innerHTML = JSON.stringify(json);
}
</script>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

