use std::fs;
use std::path::{Path, PathBuf};

pub fn execute(dir_path: &Path, youtube_file: &Option<PathBuf>) -> Result<(), String> {
    let summary_path = dir_path.join("SUMMARY.md");
    if !summary_path.exists() {
        return Err(format!(
            "Không tìm thấy tệp SUMMARY.md tại thư mục: {:?}",
            dir_path
        ));
    }

    println!("🔄 Đang phân tích tệp SUMMARY.md...");
    let summary_content =
        fs::read_to_string(&summary_path).map_err(|e| format!("Lỗi đọc SUMMARY.md: {}", e))?;

    // 1. Quét file SUMMARY.md để tìm các bài học theo cấu trúc: - [Title](filename.md)
    let mut lessons = Vec::new();
    for line in summary_content.lines() {
        let line = line.trim();
        // Cắt chuỗi siêu nhẹ không cần dùng thư viện regex
        if line.starts_with("- [")
            && let (Some(end_title_idx), Some(end_link_idx)) = (line.find("]("), line.rfind(')'))
        {
            let title = &line[3..end_title_idx];
            let filename = &line[end_title_idx + 2..end_link_idx];
            lessons.push((title.to_string(), filename.to_string()));
        }
    }

    if lessons.is_empty() {
        return Err(
            "Tệp SUMMARY.md trống hoặc sai định dạng. Yêu cầu: `- [Tên bài](file.md)`".to_string(),
        );
    }

    // 2. Xử lý map link Youtube nếu người dùng dùng cờ -y
    let mut youtube_urls = Vec::new();
    if let Some(yt_path) = youtube_file {
        let yt_content = fs::read_to_string(yt_path)
            .map_err(|e| format!("Lỗi đọc tệp chứa link Youtube: {}", e))?;

        for line in yt_content.lines() {
            let url = line.trim();
            if !url.is_empty() {
                youtube_urls.push(url.to_string());
            }
        }

        // VALIDATE: Kiểm tra tính toàn vẹn (Số dòng phải bằng nhau)
        if youtube_urls.len() != lessons.len() {
            return Err(format!(
                "SỐ LƯỢNG KHÔNG KHỚP: Có {} bài học trong SUMMARY.md nhưng lại có {} video Youtube. Vui lòng kiểm tra lại!",
                lessons.len(),
                youtube_urls.len()
            ));
        }
        println!("✅ Đã map thành công {} link Youtube.", youtube_urls.len());
    }

    // 3. Tiến hành sinh các tệp Markdown thực tế
    println!("⚙️ Tiến hành sinh tệp markdown...");
    let mut created_count = 0;
    let mut skipped_count = 0;

    for (i, (title, filename)) in lessons.iter().enumerate() {
        let file_path = dir_path.join(filename);

        let yt_metadata = if !youtube_urls.is_empty() {
            format!("\nyoutube: {}", youtube_urls[i])
        } else {
            "".to_string()
        };

        // Ghi Frontmatter
        let content = format!(
            "---\ntitle: \"{}\"{}\n---\n\nNội dung bài học **{}** sẽ được viết tại đây...\n",
            title, yt_metadata, title
        );

        // Không ghi đè nếu file đã tồn tại để tránh mất dữ liệu cũ của người dùng
        if !file_path.exists() {
            fs::write(&file_path, content)
                .map_err(|e| format!("Không thể tạo tệp {}: {}", filename, e))?;
            println!("   ➕ Đã tạo: {}", filename);
            created_count += 1;
        } else {
            println!("   ⏭️  Bỏ qua (Đã tồn tại): {}", filename);
            skipped_count += 1;
        }
    }

    println!(
        "\n🎉 Hoàn tất! Đã tạo mới {} bài học (Bỏ qua {} bài đã có sẵn).",
        created_count, skipped_count
    );
    Ok(())
}
