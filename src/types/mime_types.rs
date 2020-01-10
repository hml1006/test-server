use std::str::FromStr;
use std::string::ToString;
use std::fmt;

use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeTypeParseError(());

impl fmt::Display for MimeTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse mime type from file extension failed")
    }
}

impl Error for MimeTypeParseError {}

pub enum MimeType {
    TextHtml,
    TextCss,
    TextPlain,
    TextXml,
    TextMathml,
    TextJad,
    TextWml,
    TextHtc,

    ImageGif,
    ImageJpeg,
    ImagePng,
    ImageSvgXml,
    ImageTiff,
    ImageWebp,
    ImageVndWapWbmp,
    ImageXIcon,
    ImageXJng,
    ImageXMsBmp,

    FontWoff,
    FontWoff2,

    ApplicationJavaScript,
    ApplicationJson,
    ApplicationAtom,
    ApplicationRss,
    ApplicationJava,
    ApplicationHqx,
    ApplicationMsword,
    ApplicationPdf,
    ApplicationPostScript,
    ApplicationRtf,
    ApplicationM3u8,
    ApplicationKml,
    ApplicationKmz,
    ApplicationMsExcel,
    ApplicationMsFrontObj,
    ApplicationMsPpt,
    ApplicationOdg,
    ApplicationOdp,
    ApplicationOds,
    ApplicationOdt,
    ApplicationMsPptx,
    ApplicationMsXlsx,
    ApplicationMsDocx,
    ApplicationWmlc,
    Application7z,
    ApplicationCco,
    ApplicationJardiff,
    ApplicationJnlp,
    ApplicationRun,
    ApplicationPerl,
    ApplicationPilot,
    ApplicationRar,
    ApplicationRpm,
    ApplicationSea,
    ApplicationSwf,
    ApplicationSit,
    ApplicationTcl,
    ApplicationCert,
    ApplicationXpi,
    ApplicationXhtml,
    ApplicationXspf,
    ApplicationZip,
    ApplicationOctetStream,

    AudioMidi,
    AudioMpeg,
    AudioOgg,
    AudioM4a,
    AudioRa,

    Video3gpp,
    VideoMp2t,
    VideoMp4,
    VideoMpeg,
    VideoMov,
    VideoWebm,
    VideoFlv,
    VideoM4v,
    VideoMng,
    VideoAsf,
    VideoWmv,
    VideoAvi,
}

impl MimeType {
    pub(crate) fn is_text(&self) -> bool {
        match self {
            MimeType::TextPlain|MimeType::TextXml|MimeType::TextHtml|MimeType::TextCss|MimeType::TextMathml|
            MimeType::TextJad |MimeType::TextWml|MimeType::TextHtc => true,

            MimeType::ApplicationJavaScript|MimeType::ApplicationXhtml|MimeType::ApplicationRss|MimeType::ApplicationJson => true,

            _ => false
        }
    }
}

impl FromStr for MimeType {
    type Err = MimeTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" | "htm " | "shtml" => Ok(MimeType::TextHtml),
            "css" => Ok(MimeType::TextCss),
            "xml" => Ok(MimeType::TextXml),
            "mml" => Ok(MimeType::TextMathml),
            "txt" => Ok(MimeType::TextPlain),
            "jad" => Ok(MimeType::TextJad),
            "wml" => Ok(MimeType::TextWml),
            "htc" => Ok(MimeType::TextHtc),

            "jpeg" | "jpg" => Ok(MimeType::ImageJpeg),
            "gif" => Ok(MimeType::ImageGif),
            "png" => Ok(MimeType::ImagePng),
            "svg" | "svgz" => Ok(MimeType::ImageSvgXml),
            "tif" | "tiff" => Ok(MimeType::ImageTiff),
            "webmp" => Ok(MimeType::ImageVndWapWbmp),
            "webp" => Ok(MimeType::ImageWebp),
            "ico" => Ok(MimeType::ImageXIcon),
            "jng" => Ok(MimeType::ImageXJng),
            "bmp" => Ok(MimeType::ImageXMsBmp),

            "woff" => Ok(MimeType::FontWoff),
            "woff2" => Ok(MimeType::FontWoff2),

            "js" => Ok(MimeType::ApplicationJavaScript),
            "atom" => Ok(MimeType::ApplicationAtom),
            "rss" => Ok(MimeType::ApplicationRss),
            "jar" | "war" | "ear" => Ok(MimeType::ApplicationJava),
            "json" => Ok(MimeType::ApplicationJson),
            "hqx" => Ok(MimeType::ApplicationHqx),
            "doc" => Ok(MimeType::ApplicationMsword),
            "pdf" => Ok(MimeType::ApplicationPdf),
            "ps"|"eps"|"ai" => Ok(MimeType::ApplicationPostScript),
            "rtf" => Ok(MimeType::ApplicationRtf),
            "m3u8" => Ok(MimeType::ApplicationM3u8),
            "kml" => Ok(MimeType::ApplicationKml),
            "kmz" => Ok(MimeType::ApplicationKmz),
            "xls" => Ok(MimeType::ApplicationMsExcel),
            "eot" => Ok(MimeType::ApplicationMsFrontObj),
            "ppt" => Ok(MimeType::ApplicationMsPpt),
            "odg" => Ok(MimeType::ApplicationOdg),
            "odp" => Ok(MimeType::ApplicationOdp),
            "ods" => Ok(MimeType::ApplicationOds),
            "odt" => Ok(MimeType::ApplicationOdt),
            "pptx" => Ok(MimeType::ApplicationMsPptx),
            "xlsx" => Ok(MimeType::ApplicationMsXlsx),
            "docx" => Ok(MimeType::ApplicationMsDocx),
            "wmlc" => Ok(MimeType::ApplicationWmlc),
            "7z" => Ok(MimeType::Application7z),
            "cco" => Ok(MimeType::ApplicationCco),
            "jardiff" => Ok(MimeType::ApplicationJardiff),
            "jnlp" => Ok(MimeType::ApplicationJnlp),
            "run" => Ok(MimeType::ApplicationRun),
            "pl"|"pm" => Ok(MimeType::ApplicationPerl),
            "prc"|"pdb" => Ok(MimeType::ApplicationPilot),
            "rar" => Ok(MimeType::ApplicationRar),
            "rpm" => Ok(MimeType::ApplicationRpm),
            "sea" => Ok(MimeType::ApplicationSea),
            "swf" => Ok(MimeType::ApplicationSwf),
            "sit" => Ok(MimeType::ApplicationSit),
            "tcl"|"tk" => Ok(MimeType::ApplicationTcl),
            "der"|"perm"|"crt" => Ok(MimeType::ApplicationCert),
            "xpi" => Ok(MimeType::ApplicationXpi),
            "xhtml" => Ok(MimeType::ApplicationXhtml),
            "xspf" => Ok(MimeType::ApplicationXspf),
            "zip" => Ok(MimeType::ApplicationZip),
            "bin"|"exe"|"dll"|"deb"|"dmg"|"iso"|"img"|"msi"|"msp"|"msm"|"gz" => Ok(MimeType::ApplicationOctetStream),
            "mid"|"midi"|"kar" => Ok(MimeType::AudioMidi),
            "mp3" => Ok(MimeType::AudioMpeg),
            "ogg" => Ok(MimeType::AudioOgg),
            "m4a" => Ok(MimeType::AudioM4a),
            "ra" => Ok(MimeType::AudioRa),

            "3gpp"|"3gp" => Ok(MimeType::Video3gpp),
            "ts" => Ok(MimeType::VideoMp2t),
            "mp4" => Ok(MimeType::VideoMp4),
            "mpeg"|"mpg" => Ok(MimeType::VideoMpeg),
            "mov" => Ok(MimeType::VideoMov),
            "webm" => Ok(MimeType::VideoWebm),
            "flv" => Ok(MimeType::VideoFlv),
            "m4v" => Ok(MimeType::VideoM4v),
            "mng" => Ok(MimeType::VideoMng),
            "asx"|"asf" => Ok(MimeType::VideoAsf),
            "wmv" => Ok(MimeType::VideoWmv),
            "avi" => Ok(MimeType::VideoAvi),

            _ => Err(MimeTypeParseError(()))
        }
    }
}

impl ToString for MimeType {
    fn to_string(&self) -> String {
        match self {
            MimeType::TextHtml => "text/html",
            MimeType::TextCss => "text/css",
            MimeType::TextXml => "text/xml",
            MimeType::TextMathml => "text/mathml",
            MimeType::TextPlain => "text/plain",
            MimeType::TextHtc => "text/x-component",
            MimeType::TextJad => "text/vnd.sun.j2me.app-descriptor",
            MimeType::TextWml => "text/vnd.wap.wml",

            MimeType::ImageJpeg => "image/jpeg",
            MimeType::ImageGif => "image/gif",
            MimeType::ImagePng => "image/png",
            MimeType::ImageSvgXml => "image/svg+xml",
            MimeType::ImageTiff => "image/tiff",
            MimeType::ImageVndWapWbmp => "image/vnd.wap.wbmp",
            MimeType::ImageWebp => "image/webp",
            MimeType::ImageXIcon => "image/x-icon",
            MimeType::ImageXJng => "image/x-jng",
            MimeType::ImageXMsBmp => "image/x-ms-bmp",

            MimeType::FontWoff => "font/woff",
            MimeType::FontWoff2 => "font/woff2",

            MimeType::ApplicationJava => "application/java-archive",
            MimeType::ApplicationJavaScript => "application/javascript",
            MimeType::ApplicationAtom => "application/atom+xml",
            MimeType::ApplicationRss => "application/rss+xml",
            MimeType::ApplicationHqx => "application/mac-binhex40",
            MimeType::ApplicationMsword => "application/msword",
            MimeType::ApplicationPdf => "application/pdf",
            MimeType::ApplicationPostScript => "application/postscript",
            MimeType::ApplicationRtf => "application/rtf",
            MimeType::ApplicationM3u8 => "application/vnd.apple.mpegurl",
            MimeType::ApplicationKml => "application/vnd.google-earth.kml+xml",
            MimeType::ApplicationKmz => "application/vnd.google-earth.kmz",
            MimeType::ApplicationMsExcel => "application/vnd.ms-excel",
            MimeType::ApplicationMsFrontObj => "application/vnd.frontobject",
            MimeType::ApplicationMsPpt => "application/vnd.ms-powerpoint",
            MimeType::ApplicationOdg => "application/vnd.oasis.opendocument.graphics",
            MimeType::ApplicationOdp => "application/vnd.oasis.opendocument.presentation",
            MimeType::ApplicationOds => "application/vnd.oasis.opendocument.spreadsheet",
            MimeType::ApplicationOdt => "application/vnd.oasis.opendocument.text",
            MimeType::ApplicationMsPptx => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            MimeType::ApplicationMsXlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            MimeType::ApplicationMsDocx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            MimeType::ApplicationWmlc => "application/vnd.wap.wmlc",
            MimeType::Application7z => "application/x-7z-compressed",
            MimeType::ApplicationCco => "application/x-cocoa",
            MimeType::ApplicationJardiff => "application/x-java-archive-diff",
            MimeType::ApplicationJnlp => "application/x-java-jnlp-file",
            MimeType::ApplicationRun => "application/x-makeself",
            MimeType::ApplicationPerl => "application/x-perl",
            MimeType::ApplicationPilot => "application/x-pilot",
            MimeType::ApplicationRar => "application/x-rar-compressed",
            MimeType::ApplicationRpm => "application/x-redhat-package-manager",
            MimeType::ApplicationSea => "application/x-sea",
            MimeType::ApplicationSwf => "application/x-shockwave-flash",
            MimeType::ApplicationSit => "application/x-stuffit",
            MimeType::ApplicationTcl => "application/x-tcl",
            MimeType::ApplicationCert => "application/x-x509-ca-cert",
            MimeType::ApplicationXpi => "application/x-xpinstall",
            MimeType::ApplicationXhtml => "application/xhtml+xml",
            MimeType::ApplicationXspf => "application/xspf+xml",
            MimeType::ApplicationZip => "application/zip",
            MimeType::ApplicationOctetStream => "application/octet-stream",
            MimeType::AudioMidi => "audio/midi",
            MimeType::AudioMpeg => "audio/mpeg",
            MimeType::AudioOgg => "audio/ogg",
            MimeType::AudioM4a => "audio/x-m4a",
            MimeType::AudioRa => "audio/x-readaudio",

            MimeType::Video3gpp => "video/3gpp",
            MimeType::VideoMp2t => "video/mp2t",
            MimeType::VideoMp4 => "video/mp4",
            MimeType::VideoMpeg => "video/mpeg",
            MimeType::VideoMov => "video/quicktime",
            MimeType::VideoWebm => "video/webm",
            MimeType::VideoFlv => "video/x-flv",
            MimeType::VideoM4v => "video/x-m4v",
            MimeType::VideoMng => "video/x-mng",
            MimeType::VideoAsf => "video/x-ms-asf",
            MimeType::VideoWmv => "video/x-ms-wmv",
            MimeType::VideoAvi => "video/x-msvideo",

            _ => "application/octet-stream"
        }.parse().unwrap()
    }
}