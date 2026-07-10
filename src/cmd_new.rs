use std::path::Path;
use std::{fs, io};

/// Hàm thực thi lệnh tạo dự án mới
pub fn execute(target_path: &Path) -> io::Result<()> {
    // 1. Kiểm tra thư mục đích
    if target_path.exists() {
        // Nếu đã tồn tại, kiểm tra xem nó có trống không
        let mut entries = fs::read_dir(target_path)?;
        if entries.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Thư mục không trống! Vui lòng chỉ định một thư mục trống hoặc đường dẫn mới để tránh ghi đè dữ liệu quan trọng.",
            ));
        }
    } else {
        // Tạo thư mục gốc nếu chưa tồn tại
        fs::create_dir_all(target_path)
            .map_err(|e| io::Error::new(e.kind(), format!("Không thể tạo thư mục gốc: {}", e)))?;
    }

    println!("📂 Đang khởi tạo dự án tại: {}", target_path.display());

    // 2. Tạo file cấu hình hochanh.yml
    let config_path = target_path.join("hochanh.yml");
    let config_content = r#"# Cấu hình trang học tập
site_name: "Học Tiếng Nhật Cùng Tôi"
description: "Trang web học tập tĩnh tạo bởi hochanh"
author: "Sensei"
"#;
    fs::write(&config_path, config_content).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Không có quyền ghi file hochanh.yml: {}", e),
        )
    })?;
    println!("  ✅ Đã tạo: hochanh.yml");

    // 3. Tạo cấu trúc thư mục src/sample-course
    let course_dir = target_path.join("src").join("sample-course");
    fs::create_dir_all(&course_dir)?;
    println!("  ✅ Đã tạo thư mục khóa học: src/sample-course/");

    // 4. Tạo file SUMMARY.md
    let summary_path = course_dir.join("SUMMARY.md");
    let summary_content = r#"# Nội dung khóa học Sample

- [Bài 1: Giới thiệu hệ thống](lesson1.md)
"#;
    fs::write(&summary_path, summary_content)?;
    println!("  ✅ Đã tạo: SUMMARY.md");

    // 5. Tạo file lesson1.md với Frontmatter chuẩn
    let lesson_path = course_dir.join("lesson1.md");
    let lesson_content = r#"---
title: Bài 1 - Giới thiệu hệ thống học
description: Hướng dẫn cách sử dụng trang web để học hiệu quả nhất.
youtube: https://www.youtube.com/watch?v=1SGCu28948U
---

# Nội dung bài học

Chào mừng bạn đến với khóa học đầu tiên! 

Ở khóa học này, chúng ta sẽ làm quen với giao diện. Hãy xem video bài giảng phía trên. 
Dưới đây là một số tài liệu chuẩn bị:

* **Từ vựng 1:** 先生 (Sensei) - Giáo viên
* **Từ vựng 2:** 学生 (Gakusei) - Học sinh

> **Lưu ý:** Hãy ghi chép cẩn thận nhé!
"#;
    fs::write(&lesson_path, lesson_content)?;
    println!("  ✅ Đã tạo: lesson1.md");

    println!("🎉 Hoàn tất! Khung dự án đã sẵn sàng.");
    Ok(())
}
