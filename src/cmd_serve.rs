use std::fs::File;
use std::path::Path;
use tiny_http::{Server, Response, Header};
use crate::cmd_build; // Import module build để gọi lại

pub fn execute() -> Result<(), String> {
    // 1. Tự động chạy lệnh build trước khi start server
    println!("🔄 Đang chuẩn bị dữ liệu (Tự động chạy hochanh build)...");
    if let Err(e) = cmd_build::execute() {
        return Err(format!("Không thể start server vì build thất bại: {}", e));
    }

    // 2. Khởi tạo Local Server với tiny-http
    let addr = "127.0.0.1:3000";
    let server = Server::http(addr).map_err(|e| format!("Lỗi khởi tạo server: {}", e))?;

    println!("\n=======================================================");
    println!("🌐 Server đã khởi chạy thành công!");
    println!("👉 Vui lòng mở trình duyệt và truy cập: http://{}", addr);
    println!("🛑 Bấm Ctrl + C trên Terminal để dừng server.");
    println!("=======================================================\n");

    // 3. Vòng lặp lắng nghe và xử lý request
    for request in server.incoming_requests() {
        let mut url_path = request.url().to_string();

        // Loại bỏ các tham số query (?abc=xyz) nếu có trên URL
        if let Some(pos) = url_path.find('?') {
            url_path.truncate(pos);
        }

        // Tự động điều hướng URL kết thúc bằng '/' tới 'index.html'
        if url_path.ends_with('/') {
            url_path.push_str("index.html");
        }

        // Bỏ dấu gạch chéo đầu tiên để map với thư mục nội bộ
        let relative_path = url_path.trim_start_matches('/');
        let target_file = Path::new("site").join(relative_path);

        // 4. Đọc file và trả về cho trình duyệt
        if target_file.exists() && target_file.is_file() {
            // Xác định Content-Type cơ bản để trình duyệt hiểu CSS và HTML
            let content_type = if target_file.extension().and_then(|s| s.to_str()) == Some("css") {
                "text/css; charset=utf-8"
            } else if target_file.extension().and_then(|s| s.to_str()) == Some("html") {
                "text/html; charset=utf-8"
            } else {
                "application/octet-stream"
            };

            let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap();

            match File::open(&target_file) {
                Ok(file) => {
                    let response = Response::from_file(file).with_header(header);
                    let _ = request.respond(response);
                }
                Err(_) => {
                    let response = Response::from_string("500 Internal Server Error").with_status_code(500);
                    let _ = request.respond(response);
                }
            }
        } else {
            // Xử lý lỗi 404 nếu file không tồn tại
            println!("  [404] Không tìm thấy file: {}", url_path);
            let response = Response::from_string("404 - Trang không tồn tại (Not Found)").with_status_code(404);
            let _ = request.respond(response);
        }
    }

    Ok(())
}
