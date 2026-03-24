#[cfg(target_os = "windows")]
use anyhow::{Context, Result};
#[cfg(target_os = "windows")]
use image::RgbaImage;
#[cfg(target_os = "windows")]
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow,
    SetProcessDPIAware, SetWindowLongW, SetWindowPos, WS_CAPTION, WS_THICKFRAME,
};
#[cfg(target_os = "windows")]
use xcap::Monitor;

/// 调用此函数让用户在屏幕上框选区域，返回截取的 RgbaImage。
/// 如果用户按下 Escape 键取消，或框选区域过小，则返回 None。
#[cfg(target_os = "windows")]
pub fn capture_screen_area(fps: usize) -> Result<Option<RgbaImage>> {
    unsafe {
        let _ = SetProcessDPIAware();
    }

    // 1. 抓取屏幕并提取所有像素点
    let monitor = Monitor::all()
        .with_context(|| "无法获取显示器列表")?
        .swap_remove(0);
    let image = monitor.capture_image().with_context(|| "抓图失败")?;

    let width = image.width() as usize;
    let height = image.height() as usize;
    let raw_pixels = image.into_raw();

    // 2. 准备底层画布：高亮原图与全局变暗的底图
    let mut original_bg = vec![0u32; width * height];
    let mut dark_bg = vec![0u32; width * height];

    for (i, chunk) in raw_pixels.chunks_exact(4).enumerate() {
        let (r, g, b) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32);
        let color = (r << 16) | (g << 8) | b;
        original_bg[i] = color;
        let dark_color = ((r / 2) << 16) | ((g / 2) << 8) | (b / 2);
        dark_bg[i] = dark_color;
    }

    // 3. 打开纯粹的像素推送窗口
    let mut window = Window::new(
        "Screen Selector",
        width,
        height,
        WindowOptions {
            borderless: true,
            title: false,
            topmost: true,
            resize: false,
            transparency: true,
            none: true,
            ..WindowOptions::default()
        },
    )?;

    window
        .update_with_buffer(&dark_bg, width, height)
        .with_context(|| "更新初始黑屏画布失败")?;

    let hwnd = HWND(window.get_window_handle());
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongW(
            hwnd,
            GWL_STYLE,
            ((style as u32) & !WS_CAPTION.0 & !WS_THICKFRAME.0) as i32,
        );
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
        );
        let _ = SetForegroundWindow(hwnd);
    }

    window.set_position(0, 0);
    // 使用传入的 fps 参数控制刷新率
    window.set_target_fps(fps);

    let mut start_pos: Option<(f32, f32)> = None;
    let mut end_pos: Option<(f32, f32)> = None;
    let mut is_drawing_rectangle = false;
    let mut final_rect = None;

    let mut current_buffer = dark_bg.clone();

    // 4. 交互循环
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp);
        let left_down = window.get_mouse_down(MouseButton::Left);

        if !left_down && is_drawing_rectangle {
            if let (Some(s), Some(e)) = (start_pos, end_pos) {
                let s0 = s.0.clamp(0.0, width as f32);
                let s1 = s.1.clamp(0.0, height as f32);
                let e0 = e.0.clamp(0.0, width as f32);
                let e1 = e.1.clamp(0.0, height as f32);

                let rx = s0.min(e0) as u32;
                let ry = s1.min(e1) as u32;
                let rw = (s0 - e0).abs() as u32;
                let rh = (s1 - e1).abs() as u32;

                if rw > 5 && rh > 5 {
                    final_rect = Some((rx, ry, rw, rh));
                }
            }
            break;
        }

        if left_down {
            if !is_drawing_rectangle {
                start_pos = mouse_pos;
                is_drawing_rectangle = true;
            }
            end_pos = mouse_pos;
        }

        // 画面重绘逻辑
        current_buffer.copy_from_slice(&dark_bg);
        if is_drawing_rectangle {
            if let (Some(s), Some(e)) = (start_pos, end_pos) {
                draw_rectangle(s, e, &mut current_buffer, width, height, &original_bg);
            }
        }
        window.update_with_buffer(&current_buffer, width, height)?;
    }

    drop(window);

    // 5. 裁剪最终图像并返回
    if let Some((x, y, w, h)) = final_rect {
        let mut rgba_image = RgbaImage::from_raw(width as u32, height as u32, raw_pixels)
            .with_context(|| "解析像素失败")?;
        let cropped = image::imageops::crop(&mut rgba_image, x, y, w, h).to_image();

        // 返回截取到的图片
        Ok(Some(cropped))
    } else {
        // 用户取消或框选无效
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn draw_rectangle(
    start_pos: (f32, f32),
    end_pos: (f32, f32),
    current_buffer: &mut [u32],
    width: usize,
    height: usize,
    original_bg: &[u32],
) {
    let s0 = start_pos.0.clamp(0.0, width as f32);
    let s1 = start_pos.1.clamp(0.0, height as f32);
    let e0 = end_pos.0.clamp(0.0, width as f32);
    let e1 = end_pos.1.clamp(0.0, height as f32);

    let rx = s0.min(e0) as usize;
    let ry = s1.min(e1) as usize;
    let rw = (s0 - e0).abs() as usize;
    let rh = (s1 - e1).abs() as usize;

    for y in ry..=(ry + rh).min(height - 1) {
        let row_idx = y * width;
        let start_idx = row_idx + rx;
        let end_idx = row_idx + (rx + rw).min(width - 1);
        current_buffer[start_idx..=end_idx].copy_from_slice(&original_bg[start_idx..=end_idx]);
    }

    let border_color = 0x00_00_FF_00;
    let border_thickness = 2;

    for y in 0..border_thickness {
        if ry + y < height {
            let row_top = (ry + y) * width;
            for x in rx..=(rx + rw).min(width - 1) {
                current_buffer[row_top + x] = border_color;
            }
        }
        if ry + rh >= y && ry + rh - y < height {
            let row_bottom = (ry + rh - y) * width;
            for x in rx..=(rx + rw).min(width - 1) {
                current_buffer[row_bottom + x] = border_color;
            }
        }
    }

    for x in 0..border_thickness {
        if rx + x < width {
            for y in ry..=(ry + rh).min(height - 1) {
                current_buffer[y * width + rx + x] = border_color;
            }
        }
        if rx + rw >= x && rx + rw - x < width {
            for y in ry..=(ry + rh).min(height - 1) {
                current_buffer[y * width + rx + rw - x] = border_color;
            }
        }
    }
}
