use super::{html_to_text, parse_post_details_from_html};

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

#[test]
fn parse_post_details_extracts_title_image_and_body_from_html() {
    let html = include_str!("fixtures/rugged_neva_smoove_post.html");

    let details = parse_post_details_from_html(html);

    assert_eq!(
        details.title,
        "DJ JS-1 - GROUND ORIGINAL 2: NO SELLOUT (Ground Original/Fat Beats, 2009)"
    );
    assert_eq!(
        details.img_url.as_deref(),
        Some(
            "https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEhGZT3N4a3_TY6ltDMPyCUvQ5-qiyE4PELQhGY8hnT-sqsWxxsM10SU0liGnvVW4bmENiCykLf4dQl5dkAkLgkU_08iKdIDzMxolp-QECurdHcz7gL8hyZM-JfaIQe-_-8txtT4E_tvdCiN/w1200-h630-p-k-no-nu/GroundOriginal2.jpg"
        )
    );
    assert!(details.body.starts_with("Tanto per calmare un po' le acque"));
    assert!(details.body.contains("Da dove cominciare? Nuovamente"));
    assert!(details.body.contains("[Nota a parte: fossi in voi"));
    assert!(details.body.contains("Non ho dubbi in merito.]"));
    assert!(!details.body.contains("GroundOriginal2.jpg"));
    assert!(!details.body.contains("TreZainetti.jpg"));
    assert!(!details.body.contains("DJ JS-1 - Ground Original 2: No Sellout"));
}
