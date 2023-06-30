use std::borrow::Cow;

use image::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, LumaA};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use serde::Deserialize;
use wry::http::{Request, Response};

pub fn asset_handler(req: &Request<Vec<u8>>) -> Result<Response<Cow<'static, [u8]>>, wry::Error> {
    let uri = req.uri();

    let path = uri.path();
    let query = uri.query().unwrap_or_default().trim();
    if let Some(content_type) = mimetype(path) {
        if let Some(bytes) = content(path, query) {
            let response = Response::builder()
                .status(200)
                .header("content-type", content_type)
                .header("cache-control", "max-age=3600,private,immutable")
                .body(bytes)?;
            return Ok(response);
        }
    }

    log::error!("Resource not found: uri={uri:?}");
    let response = Response::builder()
        .status(404)
        .body(Cow::Borrowed("".as_bytes()))?;
    Ok(response)
}

fn mimetype(path: &str) -> Option<&'static str> {
    match path {
        path if path.ends_with(".css") => Some("text/css; charset=utf-8"),
        path if path.ends_with(".html") => Some("text/html; charset=utf-8"),
        path if path.ends_with(".js") => Some("text/javascript; charset=utf-8"),
        path if path.ends_with(".pdf") => Some("application/pdf"),
        path if path.ends_with(".png") => Some("image/png"),
        path if path.ends_with(".svg") => Some("image/svg+xml; charset=utf-8"),
        path if path.ends_with(".txt") => Some("text/plain; charset=utf-8"),
        path if path.ends_with(".ttf") => Some("font/ttf"),
        _ => None,
    }
}

fn content(path: &str, query: &str) -> Option<Cow<'static, [u8]>> {
    match path.trim() {
        "/" | "/index.html" => {
            let index_html = crate::index_html::IndexHtml {};
            Some(Cow::Owned(index_html.to_string().into_bytes()))
        },
        "/modern-normalize.css" => Some(Cow::Borrowed(include_bytes!("modern-normalize.css"))),
        "/style.css" => Some(Cow::Borrowed(include_bytes!("style.css"))),
        "/script.js" => Some(Cow::Borrowed(include_bytes!("script.js"))),
        "/kurzlink.svg" => svg(query),
        "/kurzlink.pdf" => pdf(query),
        "/kurzlink.png" => png(query),
        _ => None,
    }
}

#[derive(Default, Deserialize)]
struct Query {
    q: String,
}

fn svg(query: &str) -> Option<Cow<'static, [u8]>> {
    let Query { q } = serde_qs::from_str(query.trim()).ok().unwrap_or_default();
    let q = match q.trim() {
        "" => "https://www.vetmed.fu-berlin.de/",
        q => q,
    };
    let qr = match QrCode::with_error_correction_level(q.as_bytes(), EcLevel::L) {
        Ok(qr) => qr,
        Err(err) => {
            log::error!("could not generate qr code for {q:?}: {err}");
            return None;
        },
    };
    let qr = qr
        .render()
        .min_dimensions(1000, 1000)
        .light_color(svg::Color("#ffffff00"))
        .build()
        .into_bytes();
    Some(Cow::Owned(qr))
}

fn pdf(query: &str) -> Option<Cow<'static, [u8]>> {
    let Query { q } = serde_qs::from_str(query.trim()).ok().unwrap_or_default();
    let q = match q.trim() {
        "" => "https://www.vetmed.fu-berlin.de/",
        q => q,
    };
    let qr = match QrCode::with_error_correction_level(q.as_bytes(), EcLevel::L) {
        Ok(qr) => qr,
        Err(err) => {
            log::error!("could not generate qr code for {q:?}: {err}");
            return None;
        },
    };
    let svg = qr
        .render()
        .min_dimensions(1000, 1000)
        .light_color(svg::Color("#ffffff00"))
        .build();
    let pdf = svg2pdf::convert_str(&svg, svg2pdf::Options {
        dpi: 300.0,
        ..svg2pdf::Options::default()
    });
    let pdf = match pdf {
        Ok(pdf) => pdf,
        Err(err) => {
            log::error!("could not convert qr code to {q:?}: {err}");
            return None;
        },
    };
    Some(Cow::Owned(pdf))
}

fn png(query: &str) -> Option<Cow<'static, [u8]>> {
    let Query { q } = serde_qs::from_str(query.trim()).ok().unwrap_or_default();
    let q = match q.trim() {
        "" => "https://www.vetmed.fu-berlin.de/",
        q => q,
    };
    let qr = match QrCode::with_error_correction_level(q.as_bytes(), EcLevel::L) {
        Ok(qr) => qr,
        Err(err) => {
            log::error!("could not generate qr code for {q:?}: {err}");
            return None;
        },
    };
    let qr = qr
        .render()
        .min_dimensions(2480, 2480)
        .light_color(LumaA([255_u8, 0_u8]))
        .build();
    let (height, width) = qr.dimensions();
    let mut img = Vec::new();
    let err = PngEncoder::new_with_quality(&mut img, CompressionType::Fast, FilterType::Up).encode(
        qr.as_raw(),
        width,
        height,
        ColorType::La8,
    );
    if let Err(err) = err {
        log::error!("could not generate png for {q:?}: {err}");
        return None;
    }
    Some(Cow::Owned(img))
}
