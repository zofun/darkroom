use image::RgbaImage;
use std::path::Path;
use windows::core::HSTRING;
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICImagingFactory,
    WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};

pub fn decode(path: &Path) -> Result<RgbaImage, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("WIC factory 创建失败: {e}"))?;
        let path = HSTRING::from(path.as_os_str());
        let decoder = factory
            .CreateDecoderFromFilename(&path, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)
            .map_err(|e| format!("HEIC 解码失败（可能缺少 HEIF 图像扩展）: {e}"))?;
        let frame = decoder.GetFrame(0).map_err(|e| format!("读取帧失败: {e}"))?;
        let converter = factory
            .CreateFormatConverter()
            .map_err(|e| format!("创建转换器失败: {e}"))?;
        converter
            .Initialize(
                &frame,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .map_err(|e| format!("像素格式转换失败: {e}"))?;
        let mut w = 0u32;
        let mut h = 0u32;
        converter
            .GetSize(&mut w, &mut h)
            .map_err(|e| format!("读取尺寸失败: {e}"))?;
        let stride = w * 4;
        let mut buf = vec![0u8; (stride * h) as usize];
        converter
            .CopyPixels(std::ptr::null(), stride, &mut buf)
            .map_err(|e| format!("拷贝像素失败: {e}"))?;
        RgbaImage::from_raw(w, h, buf).ok_or_else(|| "像素缓冲构造失败".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn missing_file_returns_error() {
        assert!(decode(Path::new("definitely_not_here.heic")).is_err());
    }
}
