use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use std::io::Cursor;

pub struct ImageConverter;

impl DocumentConverter for ImageConverter {
    fn name(&self) -> &'static str {
        "Image"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        if let Some(mime) = &info.mime_type
            && mime.starts_with("image/")
        {
            return true;
        }
        matches!(
            info.extension.as_deref(),
            Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "heic")
        )
    }

    fn convert(&self, input: &[u8], info: &StreamInfo) -> crate::Result<ConversionResult> {
        let mut md = String::new();

        // Image description header
        if let Some(filename) = &info.filename {
            md.push_str(&format!("# Image: {filename}\n\n"));
        } else {
            md.push_str("# Image\n\n");
        }

        md.push_str(&format!("- **Size**: {} bytes\n", input.len()));

        if let Some(mime) = &info.mime_type {
            md.push_str(&format!("- **Format**: {mime}\n"));
        }

        // Try to extract EXIF data
        let cursor = Cursor::new(input);
        let mut bufreader = std::io::BufReader::new(cursor);
        match exif::Reader::new().read_from_container(&mut bufreader) {
            Ok(exif_data) => {
                md.push_str("\n## EXIF Metadata\n\n");

                let important_tags = [
                    (exif::Tag::Make, "Camera Make"),
                    (exif::Tag::Model, "Camera Model"),
                    (exif::Tag::DateTime, "Date/Time"),
                    (exif::Tag::DateTimeOriginal, "Date Taken"),
                    (exif::Tag::ExposureTime, "Exposure Time"),
                    (exif::Tag::FNumber, "F-Number"),
                    (exif::Tag::ISOSpeed, "ISO"),
                    (exif::Tag::FocalLength, "Focal Length"),
                    (exif::Tag::ImageWidth, "Width"),
                    (exif::Tag::ImageLength, "Height"),
                    (exif::Tag::Orientation, "Orientation"),
                    (exif::Tag::Software, "Software"),
                    (exif::Tag::GPSLatitude, "GPS Latitude"),
                    (exif::Tag::GPSLongitude, "GPS Longitude"),
                    (exif::Tag::ImageDescription, "Description"),
                    (exif::Tag::Artist, "Artist"),
                    (exif::Tag::Copyright, "Copyright"),
                ];

                let mut found_any = false;
                for (tag, label) in &important_tags {
                    if let Some(field) = exif_data.get_field(*tag, exif::In::PRIMARY) {
                        md.push_str(&format!(
                            "- **{label}**: {}\n",
                            field.display_value().with_unit(&exif_data)
                        ));
                        found_any = true;
                    }
                }

                if !found_any {
                    // Show all available fields
                    for field in exif_data.fields() {
                        md.push_str(&format!(
                            "- **{}**: {}\n",
                            field.tag,
                            field.display_value().with_unit(&exif_data)
                        ));
                    }
                }
            }
            Err(_) => {
                md.push_str("\n*No EXIF metadata found*\n");
            }
        }

        Ok(ConversionResult::new(md.trim_end()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_images() {
        let c = ImageConverter;
        for ext in &["jpg", "jpeg", "png", "gif", "tiff", "webp"] {
            let info = StreamInfo {
                extension: Some(ext.to_string()),
                ..Default::default()
            };
            assert!(c.accepts(&info), "should accept .{ext}");
        }
    }

    #[test]
    fn accepts_image_mime() {
        let c = ImageConverter;
        let info = StreamInfo {
            mime_type: Some("image/jpeg".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn rejects_non_image() {
        let c = ImageConverter;
        let info = StreamInfo {
            mime_type: Some("text/plain".into()),
            ..Default::default()
        };
        assert!(!c.accepts(&info));
    }
}
