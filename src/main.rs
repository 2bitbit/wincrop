use wincrop::capture_screen_area;

fn main() {
    println!("请在屏幕上框选区域...");
    // 假设刷新率为 60
    match capture_screen_area(60) {
        Ok(Some(img)) => {
            // 将抓取到的图片保存到根目录
            img.save("test_crop.png").expect("保存图片失败");
            println!("截图已成功保存到当前目录下的 test_crop.png！");
        }
        Ok(None) => println!("用户取消了框选，或框选区域过小。"),
        Err(e) => eprintln!("截图失败: {:?}", e),
    }
}
