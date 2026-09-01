//! Production boundary for public discovery sites.
//!
//! Dioxus owns rendering. This module owns what is allowed to leave the
//! repository: public origins, crawl controls, a complete sitemap, a concise
//! fact record, and executable-script policy chosen explicitly per product.

use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

type Fallible = Result<(), Box<dyn Error>>;

const DX_OUTPUT: &str = "target/dx/web/release/web/public";
const STAGED_OUTPUT: &str = ".fly-artifacts/web";
const APP_DX_OUTPUT: &str = "target/dx/app/release/web/public";
const APP_STAGED_OUTPUT: &str = ".fly-artifacts/app";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Runtime {
    Static,
    Hydrate,
}

#[derive(Debug)]
struct DiscoveryConfig {
    site_url: String,
    app_url: String,
    product_name: String,
    product_summary: String,
    social_image_url: String,
    last_modified: String,
    runtime: Runtime,
    google_verification_file: Option<String>,
}

impl DiscoveryConfig {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let product_name = product_name()?;

        let product_summary = required("PUBLIC_PRODUCT_SUMMARY")?;
        if product_summary.trim() != product_summary || !(40..=320).contains(&product_summary.len())
        {
            return Err("PUBLIC_PRODUCT_SUMMARY must be 40–320 trimmed characters".into());
        }

        let last_modified = required("PUBLIC_LAST_MODIFIED")?;
        validate_date(&last_modified)?;

        let runtime = match required("PUBLIC_DISCOVERY_RUNTIME")?.as_str() {
            "static" => Runtime::Static,
            "hydrate" => Runtime::Hydrate,
            _ => return Err("PUBLIC_DISCOVERY_RUNTIME must be `static` or `hydrate`".into()),
        };

        let google_verification_file = std::env::var("GOOGLE_SITE_VERIFICATION_FILE")
            .ok()
            .map(|value| validate_google_file(&value))
            .transpose()?;

        let site_url = public_origin("PUBLIC_SITE_URL")?;
        let social_image_url = public_asset_url("PUBLIC_SOCIAL_IMAGE_URL", &site_url)?;

        Ok(Self {
            site_url,
            app_url: public_origin("PUBLIC_APP_URL")?,
            product_name,
            product_summary,
            social_image_url,
            last_modified,
            runtime,
            google_verification_file,
        })
    }
}

pub fn stage_app() -> Fallible {
    let root = workspace_root()?;
    let source = root.join(APP_DX_OUTPUT);
    let destination = root.join(APP_STAGED_OUTPUT);
    if !source.join("index.html").is_file() {
        return Err(format!(
            "missing Dioxus app output at {}\n         run documented `dx build --package app ...` command first",
            source.display()
        )
        .into());
    }

    let site_url = public_origin("PUBLIC_SITE_URL")?;
    let api_url = public_origin("PUBLIC_API_URL")?;
    let product_name = product_name()?;
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    copy_tree(&source, &destination)?;

    let index = destination.join("index.html");
    let original = fs::read_to_string(&index)?;
    if !original.contains("noindex,nofollow,noarchive") {
        return Err("app HTML is missing production noindex policy".into());
    }
    let mut runtime_assets = BTreeMap::new();
    let transformed = replace_title(
        &externalize_runtime(&original, &mut runtime_assets),
        &format!("{product_name} — app"),
    )?;
    reject_template_placeholders(&index, &transformed)?;
    if transformed.contains("<script>") {
        return Err("app release contains an inline executable script".into());
    }
    fs::write(&index, transformed)?;
    let assets = destination.join("assets");
    fs::create_dir_all(&assets)?;
    for (name, source) in runtime_assets {
        fs::write(assets.join(name), source)?;
    }
    prune_unreferenced_assets(&destination)?;

    for required in [&site_url, &api_url, &product_name] {
        if !extension_contains(&destination, "wasm", required.as_bytes())? {
            return Err(format!(
                "app WASM is missing compiled identity `{required}`; rebuild with PUBLIC_* variables"
            )
            .into());
        }
    }
    for forbidden in ["localhost", "127.0.0.1", "example.invalid"] {
        if bundle_contains(&destination, forbidden.as_bytes())? {
            return Err(format!("app bundle contains forbidden origin `{forbidden}`").into());
        }
    }
    if product_name != "rust-v2" && fs::read_to_string(&index)?.contains("rust-v2") {
        return Err("app HTML still contains starter identity `rust-v2`".into());
    }

    fs::write(
        destination.join("robots.txt"),
        "User-agent: *\nDisallow: /\n",
    )?;
    println!("staged app at {}", destination.display());
    Ok(())
}

pub fn stage() -> Fallible {
    let root = workspace_root()?;
    let source = root.join(DX_OUTPUT);
    let destination = root.join(STAGED_OUTPUT);
    if !source.join("index.html").is_file() {
        return Err(format!(
            "missing Dioxus SSG output at {}\n         run documented `dx build ... --ssg` command first",
            source.display()
        )
        .into());
    }

    let config = DiscoveryConfig::from_env()?;
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    copy_tree(&source, &destination)?;

    let html_files = files_with_extension(&destination, "html")?;
    if html_files.is_empty() {
        return Err("SSG output contains no HTML".into());
    }

    let assets = destination.join("assets");
    fs::create_dir_all(&assets)?;
    let mut runtime_assets = BTreeMap::new();
    let mut routes = Vec::new();

    for path in html_files {
        let original = fs::read_to_string(&path)?;
        if !original.contains("application/ld+json") {
            return Err(format!("{} has no JSON-LD discovery record", path.display()).into());
        }

        let transformed = match config.runtime {
            Runtime::Static => strip_runtime(&original)?,
            Runtime::Hydrate => externalize_runtime(&original, &mut runtime_assets),
        };
        assert_release_html(&path, &transformed, &config)?;
        fs::write(&path, transformed)?;
        routes.push(route_for_html(&destination, &path)?);
    }

    if config.runtime == Runtime::Static {
        remove_script_assets(&assets)?;
    }

    for (name, source) in runtime_assets {
        fs::write(assets.join(name), source)?;
    }
    prune_unreferenced_assets(&destination)?;

    routes.sort();
    routes.dedup();
    if !routes.iter().any(|route| route == "/") {
        return Err("SSG output has no root route".into());
    }
    let home = fs::read_to_string(destination.join("index.html"))?;
    if !home.contains(&config.product_summary) {
        return Err("root HTML does not contain PUBLIC_PRODUCT_SUMMARY".into());
    }

    write_discovery_files(&destination, &config, &routes)?;
    println!(
        "staged {} routes at {} ({:?} runtime)",
        routes.len(),
        destination.display(),
        config.runtime
    );
    Ok(())
}

pub fn notify_indexnow() -> Fallible {
    let site_url = public_origin("PUBLIC_SITE_URL")?;
    let key = indexnow_key(&site_url);
    let key_location = format!("{site_url}/{key}.txt");

    let verification = Command::new("curl")
        .args(["-fsS", "--max-time", "15", &key_location])
        .output()?;
    if !verification.status.success() || String::from_utf8_lossy(&verification.stdout).trim() != key
    {
        return Err(format!("deployed IndexNow key is not readable at {key_location}").into());
    }

    let request = format!(
        "https://api.indexnow.org/indexnow?url={}&key={}&keyLocation={}",
        percent_encode(&format!("{site_url}/")),
        key,
        percent_encode(&key_location)
    );
    let status = Command::new("curl")
        .args([
            "-sS",
            "--max-time",
            "20",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            &request,
        ])
        .output()?;
    if !status.status.success() {
        return Err("IndexNow request failed before returning a status".into());
    }
    let status = String::from_utf8_lossy(&status.stdout);
    if status != "200" && status != "202" {
        return Err(format!("IndexNow returned HTTP {status}").into());
    }
    println!("IndexNow accepted {site_url}/ (HTTP {status})");
    Ok(())
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask is not inside the workspace".into())
}

fn required(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name).map_err(|_| format!("missing required environment variable {name}").into())
}

fn product_name() -> Result<String, Box<dyn Error>> {
    let value = required("PUBLIC_PRODUCT_NAME")?;
    if value.trim() != value || !(2..=80).contains(&value.len()) {
        return Err("PUBLIC_PRODUCT_NAME must be 2–80 trimmed characters".into());
    }
    Ok(value)
}

fn public_origin(name: &str) -> Result<String, Box<dyn Error>> {
    let value = required(name)?;
    parse_public_origin(name, &value)
}

fn parse_public_origin(name: &str, value: &str) -> Result<String, Box<dyn Error>> {
    if !value.starts_with("https://")
        || value.chars().any(char::is_whitespace)
        || value.contains(['?', '#'])
    {
        return Err(format!("{name} must be a public HTTPS origin").into());
    }
    let authority = value
        .strip_prefix("https://")
        .expect("scheme checked above")
        .trim_end_matches('/');
    if authority.is_empty()
        || authority.contains('/')
        || authority == "localhost"
        || authority.starts_with("localhost:")
        || authority.starts_with("127.")
        || authority.ends_with(".invalid")
        || authority.ends_with(".example")
    {
        return Err(format!("{name} must be a public HTTPS origin").into());
    }
    Ok(format!("https://{authority}"))
}

fn public_asset_url(name: &str, origin: &str) -> Result<String, Box<dyn Error>> {
    let value = required(name)?;
    parse_public_asset_url(name, origin, &value)
}

fn parse_public_asset_url(name: &str, origin: &str, value: &str) -> Result<String, Box<dyn Error>> {
    let prefix = format!("{origin}/");
    if !value.starts_with(&prefix)
        || value.len() == prefix.len()
        || value.chars().any(char::is_whitespace)
        || value.contains(['?', '#'])
    {
        return Err(format!("{name} must be a same-origin public HTTPS asset URL").into());
    }
    Ok(value.to_string())
}

fn validate_date(value: &str) -> Result<(), Box<dyn Error>> {
    let bytes = value.as_bytes();
    let shape = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !shape {
        return Err("PUBLIC_LAST_MODIFIED must use YYYY-MM-DD".into());
    }
    let month: u8 = value[5..7].parse()?;
    let day: u8 = value[8..10].parse()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("PUBLIC_LAST_MODIFIED contains an invalid month or day".into());
    }
    Ok(())
}

fn validate_google_file(value: &str) -> Result<String, Box<dyn Error>> {
    let valid = value.starts_with("google")
        && value.ends_with(".html")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if !valid {
        return Err("GOOGLE_SITE_VERIFICATION_FILE must be Google's HTML filename".into());
    }
    Ok(value.to_string())
}

fn copy_tree(source: &Path, destination: &Path) -> Fallible {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    collect_files(root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(root: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Fallible {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), extension, files)?;
        } else if entry.path().extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn bundle_contains(root: &Path, needle: &[u8]) -> Result<bool, Box<dyn Error>> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if bundle_contains(&entry.path(), needle)? {
                return Ok(true);
            }
        } else if fs::read(entry.path())?
            .windows(needle.len())
            .any(|window| window == needle)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn extension_contains(root: &Path, extension: &str, needle: &[u8]) -> Result<bool, Box<dyn Error>> {
    for path in files_with_extension(root, extension)? {
        if fs::read(path)?
            .windows(needle.len())
            .any(|window| window == needle)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn replace_title(html: &str, title: &str) -> Result<String, Box<dyn Error>> {
    let Some(start) = html.find("<title>") else {
        return Err("app HTML has no title element".into());
    };
    let body = &html[start + "<title>".len()..];
    let Some(end) = body.find("</title>") else {
        return Err("app HTML has an unclosed title element".into());
    };
    let mut output = String::with_capacity(html.len() + title.len());
    output.push_str(&html[..start + "<title>".len()]);
    output.push_str(&html_escape(title));
    output.push_str(&body[end..]);
    Ok(output)
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn remove_script_assets(assets: &Path) -> Fallible {
    for extension in ["js", "wasm"] {
        for path in files_with_extension(assets, extension)? {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn prune_unreferenced_assets(public: &Path) -> Fallible {
    let assets = public.join("assets");
    let mut candidates = Vec::new();
    collect_all_files(&assets, &mut candidates)?;
    let html_files = files_with_extension(public, "html")?;
    let mut sources: Vec<Vec<u8>> = html_files.iter().map(fs::read).collect::<Result<_, _>>()?;
    let mut reachable = BTreeSet::new();

    loop {
        let mut added = Vec::new();
        for candidate in &candidates {
            let relative = candidate.strip_prefix(&assets)?;
            if reachable.contains(relative) {
                continue;
            }
            let name = relative.to_string_lossy();
            if sources
                .iter()
                .any(|source| contains_bytes(source, name.as_bytes()))
            {
                reachable.insert(relative.to_path_buf());
                added.push(fs::read(candidate)?);
            }
        }
        if added.is_empty() {
            break;
        }
        sources.extend(added);
    }

    for candidate in candidates {
        let relative = candidate.strip_prefix(&assets)?;
        if !reachable.contains(relative) {
            fs::remove_file(candidate)?;
        }
    }
    Ok(())
}

fn collect_all_files(root: &Path, files: &mut Vec<PathBuf>) -> Fallible {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_all_files(&entry.path(), files)?;
        } else {
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(())
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn strip_runtime(html: &str) -> Result<String, Box<dyn Error>> {
    if has_client_handler(html) {
        return Err(
            "static discovery runtime selected, but SSG HTML contains a client event handler"
                .into(),
        );
    }
    let without_plain = replace_exact_scripts(html, "<script>", |_| String::new());
    let without_modules = replace_module_scripts(&without_plain, String::new);
    if without_modules.contains("<script>") || without_modules.contains("type=\"module\"") {
        return Err("static release still contains executable Dioxus runtime".into());
    }
    Ok(without_modules)
}

fn externalize_runtime(html: &str, assets: &mut BTreeMap<String, String>) -> String {
    replace_exact_scripts(html, "<script>", |source| {
        let name = format!("runtime-inline-{}.js", short_hash(source.as_bytes()));
        assets
            .entry(name.clone())
            .or_insert_with(|| source.to_string());
        format!("<script src=\"/assets/{name}\"></script>")
    })
}

fn replace_exact_scripts(
    html: &str,
    opening: &str,
    mut replacement: impl FnMut(&str) -> String,
) -> String {
    let mut output = String::with_capacity(html.len());
    let mut remaining = html;
    while let Some(start) = remaining.find(opening) {
        output.push_str(&remaining[..start]);
        let body = &remaining[start + opening.len()..];
        let Some(end) = body.find("</script>") else {
            output.push_str(&remaining[start..]);
            return output;
        };
        output.push_str(&replacement(&body[..end]));
        remaining = &body[end + "</script>".len()..];
    }
    output.push_str(remaining);
    output
}

fn replace_module_scripts(html: &str, replacement: impl Fn() -> String) -> String {
    let marker = "<script type=\"module\"";
    let mut output = String::with_capacity(html.len());
    let mut remaining = html;
    while let Some(start) = remaining.find(marker) {
        output.push_str(&remaining[..start]);
        let Some(tag_end) = remaining[start..].find('>') else {
            output.push_str(&remaining[start..]);
            return output;
        };
        let body = &remaining[start + tag_end + 1..];
        let Some(end) = body.find("</script>") else {
            output.push_str(&remaining[start..]);
            return output;
        };
        output.push_str(&replacement());
        remaining = &body[end + "</script>".len()..];
    }
    output.push_str(remaining);
    output
}

fn has_client_handler(html: &str) -> bool {
    let mut remaining = html;
    let marker = "data-node-hydration=\"";
    while let Some(start) = remaining.find(marker) {
        let value = &remaining[start + marker.len()..];
        if let Some(end) = value.find('"') {
            if value[..end].contains(':') {
                return true;
            }
            remaining = &value[end + 1..];
        } else {
            break;
        }
    }
    false
}

fn assert_release_html(path: &Path, html: &str, config: &DiscoveryConfig) -> Fallible {
    reject_template_placeholders(path, html)?;
    for forbidden in ["localhost", "127.0.0.1", "example.invalid"] {
        if html.contains(forbidden) {
            return Err(
                format!("{} contains forbidden origin `{forbidden}`", path.display()).into(),
            );
        }
    }
    if config.product_name != "rust-v2" && html.contains("rust-v2") {
        return Err(format!(
            "{} still contains starter identity `rust-v2`",
            path.display()
        )
        .into());
    }
    if !html.contains(&config.site_url)
        || !html.contains(&config.product_name)
        || !html.contains(&config.social_image_url)
    {
        return Err(format!(
            "{} is missing compiled discovery identity; rebuild with PUBLIC_* variables",
            path.display()
        )
        .into());
    }
    if config.runtime == Runtime::Hydrate && html.contains("<script>") {
        return Err(format!("{} contains an inline executable script", path.display()).into());
    }
    Ok(())
}

fn reject_template_placeholders(path: &Path, html: &str) -> Fallible {
    let rendered = without_html_comments(html);
    for placeholder in [
        "{app_title}",
        "{style_include}",
        "{script}",
        "{script_include}",
    ] {
        if rendered.contains(placeholder) {
            return Err(format!(
                "{} contains unresolved template placeholder `{placeholder}`",
                path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn without_html_comments(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut remaining = html;
    while let Some(start) = remaining.find("<!--") {
        output.push_str(&remaining[..start]);
        let comment = &remaining[start + "<!--".len()..];
        let Some(end) = comment.find("-->") else {
            return output;
        };
        remaining = &comment[end + "-->".len()..];
    }
    output.push_str(remaining);
    output
}

fn route_for_html(root: &Path, path: &Path) -> Result<String, Box<dyn Error>> {
    let relative = path.strip_prefix(root)?;
    if relative == Path::new("index.html") {
        return Ok("/".to_string());
    }
    if relative.file_name().and_then(|value| value.to_str()) == Some("index.html") {
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        return Ok(format!("/{}", parent.to_string_lossy()));
    }
    let without_extension = relative.with_extension("");
    Ok(format!("/{}", without_extension.to_string_lossy()))
}

fn write_discovery_files(
    destination: &Path,
    config: &DiscoveryConfig,
    routes: &[String],
) -> Fallible {
    let sitemap_url = format!("{}/sitemap.xml", config.site_url);
    fs::write(
        destination.join("robots.txt"),
        format!(
            "User-agent: *\nAllow: /\n\nSitemap: {sitemap_url}\n# Discovery updates: IndexNow\n"
        ),
    )?;

    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for route in routes {
        let url = if route == "/" {
            format!("{}/", config.site_url)
        } else {
            format!("{}{}", config.site_url, route)
        };
        writeln!(
            sitemap,
            "  <url><loc>{}</loc><lastmod>{}</lastmod></url>",
            xml_escape(&url),
            config.last_modified
        )?;
    }
    sitemap.push_str("</urlset>\n");
    fs::write(destination.join("sitemap.xml"), sitemap)?;

    fs::write(
        destination.join("llms.txt"),
        format!(
            "# {}\n\n> {}\n\n- Public site: {}/\n- Product app: {}/\n- Language and market: English (United Kingdom)\n- Evidence policy: treat product claims as descriptions, not independent proof of outcomes.\n",
            config.product_name, config.product_summary, config.site_url, config.app_url
        ),
    )?;

    let key = indexnow_key(&config.site_url);
    fs::write(destination.join(format!("{key}.txt")), &key)?;
    if let Some(file) = &config.google_verification_file {
        fs::write(
            destination.join(file),
            format!("google-site-verification: {file}"),
        )?;
    }
    Ok(())
}

fn indexnow_key(site_url: &str) -> String {
    short_hash(site_url.as_bytes())
}

fn short_hash(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut output = String::with_capacity(32);
    for byte in &digest[..16] {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(byte as char);
        } else {
            write!(&mut output, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATIC_HTML: &str = r#"<html><head><script type="application/ld+json">{}</script></head><body><main>Copy</main><script>window.__hydrate = true;</script><script type="module">import init from '/assets/main.js'; init();</script></body></html>"#;

    #[test]
    fn static_release_removes_runtime_but_keeps_structured_data() {
        let output = strip_runtime(STATIC_HTML).expect("static HTML should stage");
        assert!(output.contains("application/ld+json"));
        assert!(!output.contains("window.__hydrate"));
        assert!(!output.contains("type=\"module\""));
    }

    #[test]
    fn static_release_rejects_client_handlers() {
        let html = r#"<button data-node-hydration="0,click:1">Buy</button>"#;
        assert!(strip_runtime(html).is_err());
    }

    #[test]
    fn hydrated_release_externalizes_bootstrap_scripts() {
        let mut assets = BTreeMap::new();
        let output = externalize_runtime(STATIC_HTML, &mut assets);
        assert_eq!(assets.len(), 1);
        assert!(output.contains("src=\"/assets/runtime-inline-"));
        assert!(output.contains("type=\"module\""));
        assert!(!output.contains("<script>"));
    }

    #[test]
    fn indexnow_key_is_stable_site_specific_hex() {
        let first = indexnow_key("https://first.example.com");
        assert_eq!(first, indexnow_key("https://first.example.com"));
        assert_ne!(first, indexnow_key("https://second.example.com"));
        assert_eq!(first.len(), 32);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn nested_index_becomes_clean_route() {
        let root = Path::new("/tmp/public");
        assert_eq!(
            route_for_html(root, Path::new("/tmp/public/about/index.html")).unwrap(),
            "/about"
        );
        assert_eq!(
            route_for_html(root, Path::new("/tmp/public/index.html")).unwrap(),
            "/"
        );
    }

    #[test]
    fn public_origin_normalizes_one_trailing_slash_and_rejects_placeholders() {
        assert_eq!(
            parse_public_origin("URL", "https://product.test.com/").unwrap(),
            "https://product.test.com"
        );
        for value in [
            "https://",
            "https:////",
            "http://product.test.com",
            "https://localhost:4401",
            "https://product.example",
            "https://product.test.com/path",
        ] {
            assert!(parse_public_origin("URL", value).is_err(), "{value}");
        }
    }

    #[test]
    fn social_image_must_be_same_origin_public_asset() {
        let origin = "https://product.test.com";
        assert_eq!(
            parse_public_asset_url("IMAGE", origin, "https://product.test.com/og.png").unwrap(),
            "https://product.test.com/og.png"
        );
        for value in [
            "https://elsewhere.test.com/og.png",
            "https://product.test.com/",
            "https://product.test.com/og.png?version=1",
        ] {
            assert!(
                parse_public_asset_url("IMAGE", origin, value).is_err(),
                "{value}"
            );
        }
    }

    #[test]
    fn app_title_replacement_escapes_product_name() {
        assert_eq!(
            replace_title("<title>old</title>", "A & B").unwrap(),
            "<title>A &amp; B</title>"
        );
    }

    #[test]
    fn template_placeholder_in_comment_is_not_rendered_leak() {
        let html = "<!-- `{script}` is unsupported --><main>Ready</main>";
        reject_template_placeholders(Path::new("index.html"), html).unwrap();
        assert!(
            reject_template_placeholders(Path::new("index.html"), "<main>{script}</main>").is_err()
        );
    }

    #[test]
    fn asset_pruning_keeps_only_transitively_reachable_generation() {
        let root =
            std::env::temp_dir().join(format!("rust-v2-asset-prune-test-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(
            root.join("index.html"),
            r#"<script type="module" src="/assets/current.js"></script>"#,
        )
        .unwrap();
        fs::write(
            root.join("assets/current.js"),
            r#"fetch("/assets/current.wasm")"#,
        )
        .unwrap();
        fs::write(root.join("assets/current.wasm"), b"wasm").unwrap();
        fs::write(root.join("assets/stale.js"), b"stale").unwrap();
        fs::write(root.join("assets/stale.wasm"), b"stale").unwrap();

        prune_unreferenced_assets(&root).unwrap();

        assert!(root.join("assets/current.js").is_file());
        assert!(root.join("assets/current.wasm").is_file());
        assert!(!root.join("assets/stale.js").exists());
        assert!(!root.join("assets/stale.wasm").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
