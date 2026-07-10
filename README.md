# hochanh

`hochanh` là công cụ cli dùng để tạo các trang web khóa học từ các tệp Markdown và video youtube nữa.

*để khi nào ném ảnh hoặc/và trang demo lên sau*

## Installation

Để cài đặt và biên dịch `hochanh` từ mã nguồn, hãy đảm bảo bạn đã cài đặt [Rust và Cargo](https://www.rust-lang.org/tools/install). Chạy các lệnh sau trong terminal của bạn:

```bash
# Bản sao dự án về máy
git clone https://github.com/duykhanh472/hochanhh.git
cd hochanh

# Tạo binary
cargo build --release
# Hoặc chạy build.sh
chmod +x build.sh
./buildsh

# (Tùy chọn) Thêm vào hệ thống để gọi trực tiếp ở mọi nơi
cargo install --path .
```

## Usage

3 câu lệnh:

### 1. Khởi tạo một dự án khóa học mới

Sử dụng lệnh `new` kèm theo đường dẫn thư mục để tự động sinh cấu trúc thư mục mẫu, file cấu hình `hochanh.yml` và bài học đầu tiên.

```bash
hochanh new my-awesome-course

```

Cấu trúc được sinh ra bao gồm:

* **`hochanh.yml`**: Tệp cấu hình tổng của trang web (Tên trang, Tác giả, Mô tả).
* **`src/sample-course/SUMMARY.md`**: Tệp điều hướng và quản lý danh sách mục lục bài học.
* **`src/sample-course/lesson1.md`**: Bài học mẫu chứa Frontmatter (YAML) để nhập tiêu đề và link video YouTube.

### 2. Biên dịch thành trang HTML

Di chuyển vào thư mục gốc của dự án (nơi có file `hochanh.yml`) và chạy lệnh `build` để dịch toàn bộ Markdown thành HTML.

```bash
hochanh build

```

Toàn bộ mã nguồn trang web tĩnh thành phẩm sẽ được xuất ra thư mục `site/`.

### 3. Khởi chạy Local Server để xem trước

Để chạy thử nghiệm và xem trước giao diện trang học tập trực tiếp trên máy tính, hãy chạy lệnh `serve`:

```bash
hochanh serve
```

Hệ thống sẽ tự động build lại dữ liệu và khởi chạy một HTTP Server siêu nhẹ tại địa chỉ: `http://localhost:3000`.

## Directory Structure (Cấu trúc thư mục)

Để `hochanh` có thể quét và biên dịch chính xác, dự án của bạn cần tuân thủ cấu trúc sắp xếp phân cấp thư mục dưới đây. Bạn có thể tự tay tạo thêm các thư mục khóa học mới bên trong `src/` theo quy tắc này:

```text
my-awesome-course/
├── hochanh.yml                # Cấu hình chung của toàn bộ trang web
└── src/                       # Thư mục chứa toàn bộ nội dung khóa học
    ├── course-slug-1/         # Thư mục khóa học số 1 (Tên thư mục viết liền không dấu làm slug)
    │   ├── SUMMARY.md         # File cấu trúc/mục lục điều hướng bài học (Bắt buộc)
    │   ├── lesson1.md         # File nội dung bài học số 1 (Markdown + Frontmatter)
    │   └── lesson2.md         # File nội dung bài học số 2
    │
    └── course-slug-2/         # Thư mục khóa học số 2
        ├── SUMMARY.md
        └── bai-mo-dau.md

```

### Quy tắc sắp xếp nội dung:

1. **Khóa học (Course):** Mỗi thư mục con nằm trực tiếp trong `src/` được tính là một khóa học độc lập. Tên thư mục sẽ được dùng làm đường dẫn URL (Ví dụ: `src/toan-cao-cap/` sẽ sinh ra liên kết `localhost:3000/toan-cao-cap/`).
2. **Quản lý danh sách bài học (`SUMMARY.md`):** Nằm bắt buộc bên trong mỗi thư mục khóa học. File này dùng định dạng danh sách Markdown để định nghĩa thứ tự và tên hiển thị của bài học trên thanh Sidebar:
```markdown
# Mục lục khóa học

- [Bài mở đầu](bai-mo-dau.md)
- [Bài học 1: Khái niệm cơ bản](lesson1.md)

```

3. **Chi tiết bài học (`.md`):** Mỗi file bài học phải có phần cấu hình Frontmatter (YAML) nằm giữa hai cặp dấu `---` ở đầu file để khai báo Metadata và mã nhúng video:
```yaml
---
title: Bài học 1: Khái niệm cơ bản
description: Tóm tắt nội dung bài học trong 1 câu ngắn gọn.
youtube: [https://www.youtube.com/watch?v=1SGCu28948U](https://www.youtube.com/watch?v=1SGCu28948U)
---
(Nội dung bài học viết bằng Markdown bình thường ở đây...)

```

## Contributing

Nhớ cập nhật hoặc bổ sung các test case tương ứng (chạy `cargo test`) trước khi mở PR nhé.

## License

[MIT](https://choosealicense.com/licenses/mit/)
