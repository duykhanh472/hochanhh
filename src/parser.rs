use crate::youtube;
use pulldown_cmark::{Parser as MarkdownParser, html};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// =============================================================================
// 1. ĐỊNH NGHĨA CÁC CẤU TRÚC DỮ LIỆU (STRUCTS)
// =============================================================================

/// Cấu hình tổng của trang web (đọc từ hochanh.yml)
#[derive(Debug, Deserialize, Serialize)]
pub struct SiteConfig {
    pub site_name: String,
    pub description: String,
    pub author: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    // Sử dụng kiểu Mapping của serde_yaml để nhận dạng cấu trúc động
    pub course: Option<Vec<serde_yaml::Mapping>>,
}

fn default_lang() -> String {
    "en".to_string()
}

// Struct để đẩy vào Template (Tera) cho Global Sidebar
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")] // Giúp Tera phân biệt được Course và Section
pub enum GlobalSidebarItem {
    Course {
        title: String,
        slug: String,
    },
    Section {
        title: String,
        courses: Vec<CourseInfo>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseInfo {
    pub title: String,
    pub slug: String,
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
    pub name: String,
    pub slug: String,
    pub summary: Vec<CourseSection>,
    pub lessons: Vec<Lesson>,
}

/// Cấu trúc phụ để parse phần Frontmatter bằng YAML ở đầu file Markdown
#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    description: Option<String>,
    youtube: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseSection {
    pub section_title: String, // Tiêu đề Section (vd: "Ngữ pháp N2", "Chủ đề gì đó")
    pub lessons: Vec<LessonItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LessonItem {
    pub title: String,
    pub file: String,
    pub url: String, // Link html tương ứng
}

impl SiteConfig {
    // Hàm này dịch dữ liệu từ hochanh.yml sang dạng mảng Sidebar dễ dùng
    pub fn get_sidebar_menu(&self) -> Vec<GlobalSidebarItem> {
        let mut menu = Vec::new();

        if let Some(course_list) = &self.course {
            for item in course_list {
                for (key, value) in item {
                    let title = key.as_str().unwrap_or("").to_string();

                    if let Some(slug) = value.as_str() {
                        // Trường hợp 1: Là một khóa học trực tiếp (ví dụ: "JLPT N2文法": "n2-bunpo/")
                        let clean_slug = slug.trim_matches('/').to_string();
                        menu.push(GlobalSidebarItem::Course {
                            title,
                            slug: clean_slug,
                        });
                    } else if let Some(sub_list) = value.as_sequence() {
                        // Trường hợp 2: Là một Section (chứa nhiều khóa học bên trong)
                        let mut sub_courses = Vec::new();
                        for sub_item in sub_list {
                            if let Some(sub_map) = sub_item.as_mapping() {
                                for (sub_k, sub_v) in sub_map {
                                    let sub_title = sub_k.as_str().unwrap_or("").to_string();
                                    let sub_slug =
                                        sub_v.as_str().unwrap_or("").trim_matches('/').to_string();
                                    sub_courses.push(CourseInfo {
                                        title: sub_title,
                                        slug: sub_slug,
                                    });
                                }
                            }
                        }
                        menu.push(GlobalSidebarItem::Section {
                            title,
                            courses: sub_courses,
                        });
                    }
                }
            }
        }
        menu
    }
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
pub fn parse_summary(content: &str) -> Vec<CourseSection> {
    let mut sections = Vec::new();

    // Nếu file không có thẻ '#' nào ở đầu, ta tạo một section mặc định không tên
    let mut current_section = CourseSection {
        section_title: String::new(),
        lessons: Vec::new(),
    };

    for line in content.lines() {
        let line = line.trim();

        // 1. Nếu gặp Section mới (bắt đầu bằng # )
        if let Some(stripped) = line.strip_prefix("# ") {
            // Lưu section trước đó (nếu đã có bài học bên trong)
            if !current_section.lessons.is_empty() || !current_section.section_title.is_empty() {
                sections.push(current_section);
            }
            // Tạo section mới
            current_section = CourseSection {
                section_title: stripped.trim().to_string(),
                lessons: Vec::new(),
            };
        }
        // 2. Nếu gặp Link bài học (bắt đầu bằng - [ )
        else if line.starts_with("- [")
            && let (Some(end_title), Some(end_link)) = (line.find("]("), line.rfind(')'))
        {
            let title = line[3..end_title].to_string();
            let file = line[end_title + 2..end_link].to_string();
            let url = file.replace(".md", ".html"); // Map sang file HTML thành phẩm

            current_section
                .lessons
                .push(LessonItem { title, file, url });
        }
    }

    // Đẩy section cuối cùng vào danh sách
    if !current_section.lessons.is_empty() || !current_section.section_title.is_empty() {
        sections.push(current_section);
    }

    sections
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
