use std::path::PathBuf;
use clap::{Parser, Subcommand};

mod youtube;
mod cmd_new;
mod cmd_build;
mod cmd_serve;
mod parser;

/// Định nghĩa Struct chính cho CLI của công cụ `hochanh`
#[derive(Parser)]
#[command(name = "hochanh")]
#[command(author = "khong ai biet ten")]
#[command(version = "0.1.0")]
#[command(about = "Một công cụ tạo trang khóa học", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Các câu lệnh con (Subcommands) mà công cụ hỗ trợ
#[derive(Subcommand)]
enum Commands {
    /// Khởi tạo một trang học tập mới tại thư mục chỉ định
    New {
        /// Đường dẫn tới thư mục cần tạo (Ví dụ: . hoặc ./my-course-site)
        path: PathBuf,
    },
    /// Biên dịch toàn bộ nội dung trong thư mục `src` thành trang web tĩnh (nằm trong `site/`)
    Build,
    /// Chạy một local web server tại cổng 3000 để xem trước giao diện học tập
    Serve,
}

fn main() {
    // Phân tích cú pháp các đối số truyền từ dòng lệnh vào
    let cli = Cli::parse();

    // Điều hướng xử lý dựa trên câu lệnh người dùng nhập vào
    match &cli.command {
        Commands::New { path } => {
            println!("🚀 [hochanh] Đang chạy lệnh `new`...");
            // Gọi logic từ module cmd_new và xử lý lỗi
            if let Err(e) = cmd_new::execute(path) {
                eprintln!("❌ LỖI: {}", e);
                std::process::exit(1);
            }
        }        
		Commands::Build => {
            // Gọi logic build
            if let Err(e) = cmd_build::execute() {
                eprintln!("❌ LỖI BUILD: {}", e);
                std::process::exit(1);
            }
        }
		Commands::Serve => {
            // Chạy logic serve thay vì chỉ in ra màn hình
            if let Err(e) = cmd_serve::execute() {
                eprintln!("❌ LỖI SERVER: {}", e);
                std::process::exit(1);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// TOÀN BỘ LUỒNG KIỂM THỬ TÍCH HỢP (INTEGRATION TESTS)
// -----------------------------------------------------------------------------
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::net::TcpStream;
    use std::io::{Write, Read};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_full_hochanh_workflow() {
        // 1. SETUP: Tạo một thư mục môi trường giả lập bên trong `target/` để cách ly dữ liệu
        let test_env_dir = Path::new("target/hochanh_test_environment");
        if test_env_dir.exists() {
            let _ = fs::remove_dir_all(test_env_dir);
        }

        // 2. TEST LỆNH `new`: Khởi tạo khung dự án mới
        let new_res = cmd_new::execute(test_env_dir);
        assert!(new_res.is_ok(), "Lệnh `new` bị lỗi!");

        // Kiểm tra xem cấu trúc file cơ bản đã xuất hiện chưa
        assert!(test_env_dir.join("hochanh.yml").exists(), "Thiếu hochanh.yml");
        assert!(test_env_dir.join("src/sample-course/SUMMARY.md").exists(), "Thiếu SUMMARY.md");
        assert!(test_env_dir.join("src/sample-course/lesson1.md").exists(), "Thiếu lesson1.md");

        // Ghi lại thư mục gốc ban đầu của dự án và chuyển "Working Directory" vào thư mục test
        // Điều này bắt buộc vì lệnh `build` và `serve` chạy dựa trên các đường dẫn tương đối.
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_env_dir).unwrap();

        // 3. TEST LỆNH `build`: Biên dịch Markdown thành HTML tĩnh
        let build_res = cmd_build::execute();
        assert!(build_res.is_ok(), "Lệnh `build` bị lỗi!");

        // Kiểm tra thành phẩm đầu ra trong thư mục `site/`
        assert!(Path::new("site/index.html").exists(), "Không sinh ra trang chủ");
        assert!(Path::new("site/css/style.css").exists(), "Không sinh ra file CSS");
        assert!(Path::new("site/sample-course/lesson1.html").exists(), "Không dịch được bài học sang HTML");

        // 4. TEST LỆNH `serve`: Khởi chạy HTTP Server
        // Vì server chạy vòng lặp vô hạn (blocking), ta phải đẩy nó sang một Background Thread (Luồng ngầm)
        thread::spawn(|| {
            let _ = cmd_serve::execute();
        });

        // Cho server 250 mili-giây để kịp "thức dậy" và bind vào cổng 3000
        thread::sleep(Duration::from_millis(250));

        // Tạo một kết nối TCP thô (Raw TCP Client) gửi request giả lập tới localhost:3000
        let mut stream = TcpStream::connect("127.0.0.1:3000")
            .expect("Server không phản hồi hoặc cổng 3000 đang bị chiếm dụng!");
        
        // Gửi một HTTP Request đúng chuẩn tiêu chuẩn
        stream.write_all(b"GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();

        // Đọc dữ liệu phản hồi từ `tiny-http` trả về
        let mut http_response = String::new();
        stream.read_to_string(&mut http_response).unwrap();

        // XÁC THỰC KẾT QUẢ SERVER:
        // Đảm bảo phản hồi trả về mã trạng thái 200 OK và chứa từ khóa HTML mong muốn
        assert!(http_response.contains("200 OK"), "Server không trả về trạng thái thành công 200 OK!");
        assert!(http_response.contains("Danh sách khóa học"), "Nội dung trang chủ render bị sai!");

        // 5. CLEANUP: Trả môi trường làm việc về vị trí cũ và xóa thư mục rác test
        std::env::set_current_dir(original_dir).unwrap();
        let _ = fs::remove_dir_all(test_env_dir);
        
        println!("✨ [Chúc mừng] Toàn bộ hệ thống hochanh vượt qua bài kiểm tra tích hợp xuất sắc!");
    }
}
