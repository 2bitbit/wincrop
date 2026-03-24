# wincrop

`wincrop` 是一个专为 Windows 平台设计的 Rust 库，提供极简的 API，用于唤起全屏截图界面，**让用户通过鼠标框选特定区域**，并直接返回区域内截取到的图像数据。

## 功能特点

* **单一职责**：只做一件事——让用户框选屏幕区域并返回图像。
* **原生图像数据**：函数直接返回 `image::RgbaImage`，无缝对接图像处理、OCR 识别或本地保存操作。
* **自定义刷新率**：支持通过参数设置 UI 界面的刷新率（FPS），平衡性能与流畅度。
* **Windows 专属**：底层使用 Windows API 和原生窗口，去除了跨平台框架的冗余。

## 安装

`cargo add wincrop`

*注意：此 crate 仅支持 Windows 平台。在非 Windows 环境下编译时，相关函数将不可用。*

## 快速开始

调用 `capture_screen_area` 函数即可拉起截图界面。用户按下鼠标左键拖拽进行框选，松开左键完成截图。用户可以随时按下 `Escape` 键取消截图。

```rust
use wincrop::capture_screen_area;

fn main() {
    println!("请在屏幕上拖拽鼠标进行框选...");
    
    // 传入 60 作为 UI 的目标 FPS
    match capture_screen_area(60) {
        Ok(Some(image)) => {
            println!("截图成功！图像尺寸: {}x{}", image.width(), image.height());
            
            // 将截取到的图像保存到本地
            if let Err(e) = image.save("screenshot.png") {
                eprintln!("保存图片失败: {}", e);
            } else {
                println!("图片已保存为 screenshot.png");
            }
        }
        Ok(None) => {
            println!("用户按下了 Esc 取消了截图，或框选区域过小。");
        }
        Err(e) => {
            eprintln!("截图系统发生错误: {}", e);
        }
    }
}
```

## 核心 API

### `pub fn capture_screen_area(fps: usize) -> Result<Option<RgbaImage>>`

* **参数**：
  * `fps`: 截图界面的目标刷新率（例如 `60` 或 `120`）。
* **返回值**：
  * `Ok(Some(RgbaImage))`：用户成功框选并截取了有效区域。
  * `Ok(None)`：用户按下了 `Escape` 键取消，或者框选的宽度/高度不足（如误触）。
  * `Err(...)`：在获取屏幕显示器列表、捕获全屏图像或创建底层窗口时发生系统级错误。

## 依赖说明

此项目底层依赖以下 crate：
* `windows`: 用于调用 Windows 系统原生 API（如 DPI 设置、窗口置顶、去除窗口边框等）。
* `xcap`: 用于捕获显示器全屏像素。
* `minifb`: 用于渲染全屏的高亮/变暗选区界面。
* `image`: 用于图像数据的内存表示和裁剪操作。

## 许可证

本项目采用 MIT 许可证或 Apache License 2.0，详情请参阅 LICENSE 文件。