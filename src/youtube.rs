/// Trích xuất Video ID từ một URL YouTube bất kỳ
pub fn extract_video_id(url: &str) -> Option<String> {
    // 1. Xử lý định dạng rút gọn: youtu.be/VIDEO_ID
    if let Some(start) = url.find("youtu.be/") {
        let id_part = &url[start + 9..];
        // Bỏ qua các tham số như ?si=... sau ID
        let id = id_part.split('?').next().unwrap_or("");
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }

    // 2. Xử lý định dạng tiêu chuẩn: youtube.com/watch?v=VIDEO_ID
    if let Some(start) = url.find("v=") {
        let id_part = &url[start + 2..];
        // Bỏ qua các tham số như &t=... hoặc &list=... sau ID
        let id = id_part.split('&').next().unwrap_or("");
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }

    // 3. Xử lý định dạng lỡ copy sẵn link embed: youtube.com/embed/VIDEO_ID
    if let Some(start) = url.find("embed/") {
        let id_part = &url[start + 6..];
        let id = id_part.split('?').next().unwrap_or("");
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }

    // Không tìm thấy định dạng hợp lệ
    None
}

/// Chuyển đổi URL bất kỳ thành URL dạng Embed
pub fn get_embed_url(url: &str) -> Option<String> {
    let video_id = extract_video_id(url)?;
    Some(format!("https://www.youtube.com/embed/{}", video_id))
}

/// Tạo mã HTML chứa iframe với giao diện Responsive chuẩn 16:9
pub fn get_embed_html(url: &str) -> Option<String> {
    let embed_url = get_embed_url(url)?;
    
    // Sử dụng CSS inline để bọc video chuẩn 16:9 (aspect-ratio),
    // giúp khung video tự động co giãn theo màn hình điện thoại hay máy tính.
    let html = format!(
        r#"<div class="video-wrapper" style="width: 100%; aspect-ratio: 16 / 9; overflow: hidden; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); background-color: #000;">
    <iframe src="{}"
        style="width: 100%; height: 100%; border: none;"
        title="YouTube video player"
        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
        allowfullscreen>
    </iframe>
</div>"#,
        embed_url
    );
    
    Some(html)
}

// -----------------------------------------------------------------------------
// MODULE KIỂM THỬ (UNIT TESTS)
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        // Test URL chuẩn
        assert_eq!(extract_video_id("https://www.youtube.com/watch?v=1SGCu28948U").as_deref(), Some("1SGCu28948U"));
        assert_eq!(extract_video_id("http://youtube.com/watch?v=1SGCu28948U").as_deref(), Some("1SGCu28948U"));

        // Test URL có chứa thêm biến thời gian hoặc danh sách
        assert_eq!(extract_video_id("https://www.youtube.com/watch?v=1SGCu28948U&t=120s").as_deref(), Some("1SGCu28948U"));
        assert_eq!(extract_video_id("https://www.youtube.com/watch?v=1SGCu28948U&list=PLxyz").as_deref(), Some("1SGCu28948U"));

        // Test URL rút gọn từ nút Share (youtu.be)
        assert_eq!(extract_video_id("https://youtu.be/1SGCu28948U").as_deref(), Some("1SGCu28948U"));
        assert_eq!(extract_video_id("https://youtu.be/1SGCu28948U?si=yWS4-Cwon98ivVzB").as_deref(), Some("1SGCu28948U"));

        // Test URL phiên bản mobile (m.youtube.com)
        assert_eq!(extract_video_id("https://m.youtube.com/watch?v=1SGCu28948U").as_deref(), Some("1SGCu28948U"));

        // Test link đã là embed sẵn
        assert_eq!(extract_video_id("https://www.youtube.com/embed/1SGCu28948U?si=abc").as_deref(), Some("1SGCu28948U"));

        // Test link rác/không hợp lệ
        assert_eq!(extract_video_id("https://google.com"), None);
        assert_eq!(extract_video_id("not a url"), None);
    }

    #[test]
    fn test_get_embed_url() {
        assert_eq!(
            get_embed_url("https://youtu.be/1SGCu28948U?si=abc").as_deref(),
            Some("https://www.youtube.com/embed/1SGCu28948U")
        );
    }
}