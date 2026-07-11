use crate::parser::{self, Course, Lesson};
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

pub fn execute() -> Result<(), String> {
    println!("Bắt đầu tạo trang...");

    // 1. Đọc file cấu hình tổng
    let config_path = Path::new("hochanh.yml");
    if !config_path.exists() {
        return Err("Tệp hochanh.yml đâu rồi bạn ơi?".to_string());
    }
    let config = parser::parse_config(config_path)?;

    // 2. Khởi tạo thư mục đích `site/` và `site/css/`
    let site_dir = Path::new("site");
    if site_dir.exists() {
        fs::remove_dir_all(site_dir)
            .map_err(|e| format!("Không thể xóa thư mục site cũ: {}", e))?;
    }
    fs::create_dir_all(site_dir.join("css"))
        .map_err(|e| format!("Lỗi tạo thư mục site/css: {}", e))?;

    // 3. Khởi tạo Template Engine (Tera) với các giao diện mặc định
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", INDEX_TEMPLATE).unwrap();
    tera.add_raw_template("course.html", COURSE_TEMPLATE)
        .unwrap();
    tera.add_raw_template("lesson.html", LESSON_TEMPLATE)
        .unwrap();

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
                // 1. Phải đọc file ra String trước
                let summary_content = fs::read_to_string(&summary_path)
                    .map_err(|e| format!("Lỗi đọc file SUMMARY: {}", e))?;

                // 2. Truyền &summary_content vào và xóa dấu ? ở cuối
                let summary = parser::parse_summary(&summary_content);
                let mut lessons: Vec<Lesson> = Vec::new();

                // Lặp qua từng section rồi mới lặp qua từng bài học
                for section in &summary {
                    for item in &section.lessons {
                        let lesson_path = path.join(&item.file);
                        if lesson_path.exists() {
                            let lesson = parser::parse_lesson(&lesson_path)?;
                            lessons.push(lesson);
                        } else {
                            println!(
                                "⚠️ Cảnh báo: Không tìm thấy file {} được nhắc đến trong SUMMARY.md",
                                item.file
                            );
                        }
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
    context.insert("global_menu", &config.get_sidebar_menu());
    let index_html = tera
        .render("index.html", &context)
        .map_err(|e| format!("Lỗi render trang chủ: {}", e))?;
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

    println!("Xong, kiểm tra thư mục `site/` xem bạn ơi.");
    Ok(())
}

// =============================================================================
// HTML / CSS TEMPLATES (Nhúng trực tiếp để chạy được ngay)
// =============================================================================

const CSS_STYLES: &str = r#"
/* Sử dụng System Fonts và tối ưu hiển thị */
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; margin: 0; padding: 0; background: #f9f9fa; color: #333; line-height: 1.6; }
a { text-decoration: none; color: #2563eb; }
a:hover { text-decoration: underline; }

/* Header chung */
.site-header { background: #fff; padding: 1rem 1.5rem; border-bottom: 1px solid #e5e7eb; box-shadow: 0 1px 2px rgba(0,0,0,0.05); }
.site-header h2 { margin: 0; font-size: 1.25rem; color: #111827; }
.site-header h2 a { color: inherit; }

/* Container & Grid (Trang chủ & Khóa học) */
.container { max-width: 1200px; margin: 0 auto; padding: 2rem 1.5rem; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1.5rem; }
.card { background: #fff; border: 1px solid #e5e7eb; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); transition: transform 0.2s; }
.card:hover { transform: translateY(-2px); box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
.card h3 { margin: 0 0 0.5rem 0; font-size: 1.1rem; }

/* Breadcrumb & Section */
.breadcrumb { margin-bottom: 1.5rem; font-size: 0.95rem; color: #6b7280; }
.breadcrumb a { color: #4b5563; }
.breadcrumb a:hover { color: #2563eb; }
.section-header { grid-column: 1 / -1; margin-top: 1rem; border-bottom: 2px solid #e5e7eb; padding-bottom: 0.5rem; font-size: 1.25rem; }
.section-title { font-size: 1.05rem; font-weight: 600; margin: 1.5rem 0 0.5rem 0; color: #111827; }

/* Section List (Sidebar & Course outline) */
ul.section-list { list-style: none; padding: 0; margin: 0; }
ul.section-list li { margin-bottom: 0.25rem; }
ul.section-list a { display: block; padding: 0.6rem 0.75rem; border-radius: 6px; color: #4b5563; transition: all 0.2s; }
ul.section-list a:hover { background: #f3f4f6; color: #111827; text-decoration: none; }
ul.section-list a.active { background: #e0e7ff; color: #2563eb; font-weight: 600; border-left: 4px solid #2563eb; padding-left: calc(0.75rem - 4px); }

/* Responsive Topbar (Giống ảnh eboard) */
.lesson-topbar { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1.5rem; background: #fff; border-bottom: 1px solid #e5e7eb; }
.back-link { font-weight: 600; color: #374151; font-size: 1rem; display: flex; align-items: center; gap: 0.25rem; }
.back-link:hover { color: #2563eb; text-decoration: none; }
.menu-toggle { background: none; border: 1px solid #e5e7eb; border-radius: 4px; padding: 0.25rem 0.5rem; font-size: 0.8rem; cursor: pointer; color: #374151; display: flex; flex-direction: column; align-items: center; }
.menu-toggle-icon { font-size: 1.1rem; line-height: 1.2; }

/* Layout trang bài học */
.lesson-layout { display: flex; min-height: calc(100vh - 120px); position: relative; }

/* Left Menu (Off-canvas cho Mobile) */
.sidebar { background: #fff; position: fixed; top: 0; left: 0; bottom: 0; width: 280px; transform: translateX(-100%); transition: transform 0.3s ease; z-index: 1000; display: flex; flex-direction: column; box-shadow: 2px 0 8px rgba(0,0,0,0.1); }
.sidebar.open { transform: translateX(0); }
.sidebar-header { display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; border-bottom: 1px solid #e5e7eb; }
.sidebar-header h3 { margin: 0; font-size: 1.1rem; }
.close-btn { background: none; border: none; font-size: 1.5rem; cursor: pointer; color: #6b7280; line-height: 1; padding: 0; }
.sidebar-content { padding: 1rem 1.5rem; overflow-y: auto; flex: 1; }

.overlay { display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.4); z-index: 999; backdrop-filter: blur(2px); }
.overlay.open { display: block; }

/* Nội dung bài giảng */
.content { flex: 1; padding: 1.5rem; max-width: 800px; margin: 0 auto; background: #fff; width: 100%; box-sizing: border-box; }
.lesson-main-title { font-size: 1.5rem; margin-top: 0; margin-bottom: 1rem; display: flex; align-items: center; gap: 0.5rem; color: #111827; }
.lesson-main-title::before { content: "▶"; font-size: 1.2rem; color: #4b5563; }

/* Video Iframe */
.video-container { position: relative; padding-bottom: 56.25%; height: 0; overflow: hidden; margin-bottom: 2rem; border-radius: 8px; background: #000; }
.video-container iframe { position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: none; }
.markdown-body img { max-width: 100%; height: auto; border-radius: 4px; }

/* PC Layout: Khóa cứng Viewport để kích hoạt cuộn độc lập 2 bên */
@media (min-width: 768px) {
    body.lesson-page { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
    body.lesson-page .site-header { padding: 1rem 2rem; flex-shrink: 0; }
    body.lesson-page .lesson-topbar { padding: 1rem 2rem; flex-shrink: 0; }
    body.lesson-page .menu-toggle { display: none; }
    
    body.lesson-page .lesson-layout { flex: 1; overflow: hidden; }
    body.lesson-page .sidebar { position: static; transform: none; width: 320px; z-index: 1; box-shadow: none; border-right: 1px solid #e5e7eb; height: 100%; }
    body.lesson-page .sidebar-header { display: none; }
    body.lesson-page .overlay { display: none !important; }
    
    body.lesson-page .content { height: 100%; overflow-y: auto; padding: 2rem 3rem; margin: 0; max-width: none; }
}
"#;

const INDEX_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ config.site_name }}</title>
    <link rel="stylesheet" href="css/style.css">
    <link rel="icon" href="data:,">
</head>
<body>
    <header class="site-header">
        <h2><a href="index.html">{{ config.site_name }}</a></h2>
    </header>
    <div class="container">
        <h1>Danh sách khóa học</h1>
        <div class="grid">
            {% for item in global_menu %}
                {% if item.type == "Course" %}
                    <div class="card">
                        <h3><a href="{{ item.slug }}/index.html">{{ item.title }}</a></h3>
                    </div>
                {% elif item.type == "Section" %}
                    <h2 class="section-header">{{ item.title }}</h2>
                    {% for course in item.courses %}
                        <div class="card">
                            <h3><a href="{{ course.slug }}/index.html">{{ course.title }}</a></h3>
                        </div>
                    {% endfor %}
                {% endif %}
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
    <link rel="stylesheet" href="../css/style.css">
    <link rel="icon" href="data:,">
</head>
<body>
    <header class="site-header">
        <h2><a href="../index.html">{{ config.site_name }}</a></h2>
    </header>
    <div class="container">
        <div class="breadcrumb"><a href="../index.html">Trang chủ</a> / {{ course.name }}</div>
        <h1>Lộ trình khóa học: {{ course.name }}</h1>
        <div class="timeline">
            {% for section in course.summary %}
                {% if section.section_title != "" %}
                    <h3 class="section-title">{{ section.section_title }}</h3>
                {% endif %}
                <ul class="section-list">
                    {% for lesson in section.lessons %}
                    <li>
                        <a href="{{ lesson.url }}">
                            <strong>{{ lesson.title }}</strong>
                        </a>
                    </li>
                    {% endfor %}
                </ul>
            {% endfor %}
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
    <link rel="stylesheet" href="../css/style.css">
    <link rel="icon" href="data:,">
</head>
<body class="lesson-page">
    <header class="site-header">
        <h2><a href="../index.html">{{ config.site_name }}</a></h2>
    </header>

    <!-- Thanh Topbar điều hướng tương đối -->
    <div class="lesson-topbar">
        <a href="index.html" class="back-link">❮ {{ course.name }}</a>
        <button class="menu-toggle" onclick="toggleMenu()">
            <span class="menu-toggle-icon">☰</span>
            Mục lục
        </button>
    </div>

    <div class="lesson-layout">
        <!-- Overlay nền tối trên Mobile -->
        <div class="overlay" id="overlay" onclick="toggleMenu()"></div>

        <!-- Left Menu (Hỗ trợ cuộn độc lập) -->
        <aside class="sidebar" id="sidebar">
            <div class="sidebar-header">
                <h3>Nội dung khóa học</h3>
                <button class="close-btn" onclick="toggleMenu()">×</button>
            </div>
            <div class="sidebar-content" id="sidebar-content">
                {% for section in course.summary %}
                    {% if section.section_title != "" %}
                        <h4 class="section-title">{{ section.section_title }}</h4>
                    {% endif %}
                    <ul class="section-list">
                        {% for item in section.lessons %}
                        <li>
                            <a href="{{ item.url }}" class="{% if item.url == lesson.file_name %}active{% endif %}">
                                {{ item.title }}
                            </a>
                        </li>
                        {% endfor %}
                    </ul>
                {% endfor %}
            </div>
        </aside>

        <!-- Nội dung bài học chính -->
        <main class="content">
            <h1 class="lesson-main-title">{{ lesson.title }}</h1>
            
            {% if lesson.youtube_html %}
            <div class="video-container">
                {{ lesson.youtube_html | safe }}
            </div>
            {% endif %}
            
            <div class="markdown-body">
                {{ lesson.content_html | safe }}
            </div>
        </main>
    </div>

    <script>
        // Đóng mở menu trên thiết bị di động
        function toggleMenu() {
            document.getElementById('sidebar').classList.toggle('open');
            document.getElementById('overlay').classList.toggle('open');
        }

        // Tự động cuộn phần Sidebar đến vị trí bài học đang Active
        document.addEventListener("DOMContentLoaded", function() {
            const activeLesson = document.querySelector('.sidebar a.active');
            if (activeLesson) {
                // Cuộn mượt đưa phần tử active vào giữa tầm nhìn của sidebar-content
                activeLesson.scrollIntoView({ block: 'center', behavior: 'instant' });
            }
        });
    </script>
</body>
</html>
"#;
