use std::fs;
use std::path::Path;
use tera::{Tera, Context};
use crate::parser::{self, Course, Lesson};

pub fn execute() -> Result<(), String> {
    println!("🏗️  Bắt đầu quá trình build trang tĩnh...");

    // 1. Đọc file cấu hình tổng
    let config_path = Path::new("hochanh.yml");
    if !config_path.exists() {
        return Err("Không tìm thấy file hochanh.yml. Hãy chắc chắn bạn đang ở thư mục gốc của dự án.".to_string());
    }
    let config = parser::parse_config(config_path)?;

    // 2. Khởi tạo thư mục đích `site/` và `site/css/`
    let site_dir = Path::new("site");
    if site_dir.exists() {
        fs::remove_dir_all(site_dir).map_err(|e| format!("Không thể xóa thư mục site cũ: {}", e))?;
    }
    fs::create_dir_all(site_dir.join("css")).map_err(|e| format!("Lỗi tạo thư mục site/css: {}", e))?;

    // 3. Khởi tạo Template Engine (Tera) với các giao diện mặc định
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", INDEX_TEMPLATE).unwrap();
    tera.add_raw_template("course.html", COURSE_TEMPLATE).unwrap();
    tera.add_raw_template("lesson.html", LESSON_TEMPLATE).unwrap();

    // 4. Quét thư mục `src/` để tìm các khóa học
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        return Err("Không tìm thấy thư mục src/ chứa nội dung.".to_string());
    }

    let mut courses: Vec<Course> = Vec::new();

    for entry in fs::read_dir(src_dir).map_err(|e| format!("Lỗi đọc src/: {}", e))? {
        let entry = entry.unwrap();
        let path = entry.path();

        // Nếu là thư mục, ta coi nó là một Khóa học
        if path.is_dir() {
            let slug = entry.file_name().to_string_lossy().to_string();
            let summary_path = path.join("SUMMARY.md");

            if summary_path.exists() {
                let summary = parser::parse_summary(&summary_path)?;
                let mut lessons: Vec<Lesson> = Vec::new();

                // Lặp qua từng bài học trong SUMMARY.md để parse
                for sum in &summary {
                    let lesson_path = path.join(&sum.file_path);
                    if lesson_path.exists() {
                        let lesson = parser::parse_lesson(&lesson_path)?;
                        lessons.push(lesson);
                    } else {
                        println!("⚠️ Cảnh báo: Không tìm thấy file {} được nhắc đến trong SUMMARY.md", sum.file_path);
                    }
                }

                courses.push(Course {
                    name: slug.clone(), // Tạm dùng slug làm tên, có thể làm đẹp hơn sau
                    slug,
                    summary,
                    lessons,
                });
            }
        }
    }

    // 5. RENDER GIAO DIỆN

    // Tạo file CSS tĩnh
    fs::write(site_dir.join("css").join("style.css"), CSS_STYLES)
        .map_err(|e| format!("Lỗi ghi file CSS: {}", e))?;

    // Render Trang chủ (index.html)
    let mut context = Context::new();
    context.insert("config", &config);
    context.insert("courses", &courses);
    let index_html = tera.render("index.html", &context).map_err(|e| format!("Lỗi render trang chủ: {}", e))?;
    fs::write(site_dir.join("index.html"), index_html).unwrap();
    println!("  ✅ Render: Trang chủ (index.html)");

    // Render các trang Khóa học và Bài học
    for course in &courses {
        let course_dir = site_dir.join(&course.slug);
        fs::create_dir_all(&course_dir).unwrap();

        // Trang danh sách bài học của khóa
        let mut ctx = Context::new();
        ctx.insert("config", &config);
        ctx.insert("course", course);
        let course_html = tera.render("course.html", &ctx).unwrap();
        fs::write(course_dir.join("index.html"), course_html).unwrap();
        println!("  ✅ Render Khóa học: {}", course.slug);

        // Các trang chi tiết bài học
        for lesson in &course.lessons {
            let mut ctx_lesson = Context::new();
            ctx_lesson.insert("config", &config);
            ctx_lesson.insert("course", course);
            ctx_lesson.insert("lesson", lesson);
            let lesson_html = tera.render("lesson.html", &ctx_lesson).unwrap();
            fs::write(course_dir.join(&lesson.file_name), lesson_html).unwrap();
            println!("    - Bài học: {}", lesson.file_name);
        }
    }

    println!("🎉 Hoàn tất! Mã nguồn đã được build thành công ra thư mục `site/`.");
    Ok(())
}

// =============================================================================
// HTML / CSS TEMPLATES (Nhúng trực tiếp để chạy được ngay)
// =============================================================================

const CSS_STYLES: &str = r#"
/* Sử dụng System Fonts và tối ưu hiển thị như yêu cầu */
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; margin: 0; padding: 0; background: #f9f9fa; color: #333; line-height: 1.6; }
a { text-decoration: none; color: #2563eb; }
a:hover { text-decoration: underline; }
header { background: #fff; padding: 1rem 2rem; border-bottom: 1px solid #e5e7eb; box-shadow: 0 1px 2px rgba(0,0,0,0.05); }
.container { max-width: 1200px; margin: 0 auto; padding: 2rem; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1.5rem; }
.card { background: #fff; border: 1px solid #e5e7eb; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }

/* Layout trang bài học */
.lesson-layout { display: flex; min-height: calc(100vh - 60px); }
.sidebar { width: 300px; background: #fff; border-right: 1px solid #e5e7eb; padding: 1.5rem; }
.sidebar ul { list-style: none; padding: 0; }
.sidebar li { margin-bottom: 0.5rem; }
.sidebar a { display: block; padding: 0.5rem; border-radius: 4px; color: #4b5563; }
.sidebar a:hover { background: #f3f4f6; text-decoration: none; }
.content { flex: 1; padding: 2rem; max-width: 800px; margin: 0 auto; background: #fff; }
.video-container { margin-bottom: 2rem; }
"#;

const INDEX_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ config.site_name }}</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <header><h2>{{ config.site_name }}</h2></header>
    <div class="container">
        <h1>Danh sách khóa học</h1>
        <div class="grid">
            {% for course in courses %}
            <div class="card">
                <h3><a href="/{{ course.slug }}/">{{ course.name }}</a></h3>
                <p>Số bài học: {{ course.lessons | length }}</p>
            </div>
            {% endfor %}
        </div>
    </div>
</body>
</html>
"#;

const COURSE_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ course.name }} - {{ config.site_name }}</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <header><h2><a href="/">{{ config.site_name }}</a> / {{ course.name }}</h2></header>
    <div class="container">
        <h1>Lộ trình khóa học: {{ course.name }}</h1>
        <div class="timeline">
            <ul>
                {% for sum in course.summary %}
                <li>
                    <a href="{{ sum.file_path | replace(from=".md", to=".html") }}">
                        <strong>{{ sum.title }}</strong>
                    </a>
                </li>
                {% endfor %}
            </ul>
        </div>
    </div>
</body>
</html>
"#;

const LESSON_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ lesson.title }} - {{ config.site_name }}</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <header><h2><a href="/">{{ config.site_name }}</a> / <a href="/{{ course.slug }}/">{{ course.name }}</a></h2></header>
    <div class="lesson-layout">
        <aside class="sidebar">
            <h3>Nội dung khóa học</h3>
            <ul>
                {% for sum in course.summary %}
                <li>
                    <a href="{{ sum.file_path | replace(from=".md", to=".html") }}">{{ sum.title }}</a>
                </li>
                {% endfor %}
            </ul>
        </aside>
        <main class="content">
            {% if lesson.youtube_html %}
            <div class="video-container">
                {{ lesson.youtube_html | safe }}
            </div>
            {% endif %}
            <h1>{{ lesson.title }}</h1>
            <div class="markdown-body">
                {{ lesson.content_html | safe }}
            </div>
        </main>
    </div>
</body>
</html>
"#;
