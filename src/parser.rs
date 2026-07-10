use crate::youtube;
use pulldown_cmark::{Parser as MarkdownParser, html};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// =============================================================================
// 1. ĐỊNH NGHĨA CÁC CẤU TRÚC DỮ LIỆU (STRUCTS)
// =============================================================================

/// Cấu hình tổng của trang web (đọc từ hochanh.yml)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteConfig {
    pub site_name: String,
    pub description: String,
    pub author: String,
}

/// Thông tin tóm tắt của một bài học (đọc từ SUMMARY.md)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LessonSummary {
    pub title: String,
    pub file_path: String, // Ví dụ: "lesson1.md"
}

/// Dữ liệu đầy đủ của một bài học sau khi đã parse xong xuôi
#[derive(Debug, Serialize, Clone)]
pub struct Lesson {
    pub title: String,
    pub description: String,
    pub youtube_url: Option<String>,  // Link gốc người dùng nhập
    pub youtube_html: Option<String>, // Mã nhúng <iframe> responsive tự động tạo
    pub content_html: Option<String>, // Nội dung Markdown đã biến thành HTML
    pub file_name: String,            // Tên file để xuất ra HTML (Ví dụ: "lesson1.html")
}

/// Cấu trúc tổng thể của một Khóa học (Ví dụ: jlpt-n1)
#[derive(Debug, Serialize, Clone)]
pub struct Course {
    pub name: String, // Tên khóa học lấy từ tên thư mục (Ví dụ: "jlpt-n1")
    pub slug: String, // Đường dẫn URL (Ví dụ: "jlpt-n1")
    pub summary: Vec<LessonSummary>, // Danh sách bài học lấy từ SUMMARY.md
    pub lessons: Vec<Lesson>, // Chi tiết nội dung của từng bài học trong khóa
}

/// Cấu trúc phụ để parse phần Frontmatter bằng YAML ở đầu file Markdown
#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    description: Option<String>,
    youtube: Option<String>,
}

// =============================================================================
// 2. CÁC HÀM XỬ LÝ PARSE DỮ LIỆU
// =============================================================================

/// Nhiệm vụ 1: Đọc và parse file cấu hình hochanh.yml
pub fn parse_config<P: AsRef<Path>>(path: P) -> Result<SiteConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Không thể đọc file config hochanh.yml: {}", e))?;

    let config: SiteConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Định dạng file hochanh.yml không hợp lệ: {}", e))?;

    Ok(config)
}

/// Nhiệm vụ 2: Đọc file SUMMARY.md để lấy danh sách bài học
/// Hỗ trợ định dạng chuẩn: `- [Tên bài học](tên_file.md)`
pub fn parse_summary<P: AsRef<Path>>(path: P) -> Result<Vec<LessonSummary>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Không thể đọc file SUMMARY.md: {}", e))?;

    let mut summaries = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue; // Bỏ qua dòng trống hoặc tiêu đề lớn h1 trong file summary
        }

        if trimmed.starts_with("- [") && trimmed.contains("](") && trimmed.ends_with(')') {
            if let (Some(start_title), Some(end_title), Some(start_link)) =
                (trimmed.find('['), trimmed.find(']'), trimmed.find('('))
            {
                let title = trimmed[start_title + 1..end_title].to_string();
                let file_path = trimmed[start_link + 1..trimmed.len() - 1].to_string();

                summaries.push(LessonSummary { title, file_path });
            }
        } else {
            println!(
                "⚠️  [Cảnh báo] Dòng số {} trong SUMMARY.md sai cú pháp mẫu `- [Title](file.md)`",
                line_num + 1
            );
        }
    }

    Ok(summaries)
}

/// Nhiệm vụ 3: Tách Frontmatter, parse Markdown thành HTML, tự tạo link Embed Youtube
pub fn parse_lesson<P: AsRef<Path>>(path: P) -> Result<Lesson, String> {
    let file_name = path
        .as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.replace(".md", ".html"))
        .unwrap_or_else(|| "lesson.html".to_string());

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Không thể đọc file bài học {:?}: {}", path.as_ref(), e))?;

    // Tách phần Frontmatter nằm giữa hai cặp dấu `---`
    if !content.starts_with("---") {
        return Err(format!(
            "File bài học {:?} thiếu phần Frontmatter (---) ở đầu file!",
            path.as_ref()
        ));
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(format!(
            "Cấu trúc Frontmatter trong file {:?} không đóng đủ hai cặp '---'",
            path.as_ref()
        ));
    }

    let frontmatter_raw = parts[1];
    let markdown_body = parts[2];

    // Parse phần Frontmatter YAML
    let fm: Frontmatter = serde_yaml::from_str(frontmatter_raw)
        .map_err(|e| format!("Lỗi cấu trúc YAML tại file {:?}: {}", path.as_ref(), e))?;

    // Xử lý tự động sinh mã nhúng YouTube dựa trên module youtube.rs (Bước 2)
    let youtube_html = fm
        .youtube
        .as_ref()
        .and_then(|url| youtube::get_embed_html(url));

    // Dùng pulldown-cmark chuyển đổi Markdown Body thành HTML chuỗi
    let mut content_html = String::new();
    let md_parser = MarkdownParser::new(markdown_body);
    html::push_html(&mut content_html, md_parser);

    Ok(Lesson {
        title: fm.title,
        description: fm.description.unwrap_or_default(),
        youtube_url: fm.youtube,
        youtube_html,
        content_html: Some(content_html),
        file_name,
    })
}
