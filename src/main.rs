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
#[command(author = "hochanh team")]
#[command(version = "0.1.0")]
#[command(about = "Static Site Generator dành cho các khóa học trực tuyến", long_about = None)]
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
