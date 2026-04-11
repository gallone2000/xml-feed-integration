use super::html_to_text;

#[test]
fn html_to_text_replaces_common_html_entities() {
    let html = r#"<p>&amp; &lt; &gt; &quot; &#39; &nbsp;</p>"#;

    let text = html_to_text(html);

    assert_eq!(text, "& < > \" '");
}

#[test]
fn html_to_text_replaces_numeric_special_entities() {
    let html = r#"<p>&#171;ciao&#187; &#8216;test&#8217; &#8220;quote&#8221;</p>"#;

    let text = html_to_text(html);

    assert_eq!(text, "«ciao» ‘test’ “quote”");
}
