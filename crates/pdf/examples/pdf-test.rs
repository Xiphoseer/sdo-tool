use pdf::{file::Storage, object::Resolve, object::Stream, primitive::Dictionary};

fn _test(trailer: Dictionary, storage: Storage<Vec<u8>>) {
    println!("Trailer");
    let mut root_ref = None;
    let mut info_ref = None;

    for (key, value) in &trailer {
        println!("{}: {}", key, value);
        match key.as_str() {
            "Root" => {
                root_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expect `Root` to be reference"),
                );
            }
            "Info" => {
                info_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expect `Info` to be reference"),
                );
            }
            _ => {}
        }
    }
    let root_ref = root_ref.expect("Expected `Root` in trailer");
    let info_ref = info_ref.expect("Expected `Info` in trailer");
    println!("root_ref: {:?}", root_ref);
    println!("info_ref: {:?}", info_ref);

    let root = storage
        .resolve(root_ref)
        .expect("Expected `Root` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Root` to be a dictionary");
    let info = storage
        .resolve(info_ref)
        .expect("Expected `Info` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Info` to be a dictionary");
    println!("root: {:?}", root);
    println!("info: {:?}", info);

    let mut pages_ref = None;
    let mut metadata_ref = None;
    for (key, value) in &root {
        println!("{}: {}", key, value);
        match key.as_str() {
            "Pages" => {
                pages_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expected `Pages` to be a reference"),
                );
            }
            "Metadata" => {
                metadata_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expected `Metadata` to be a reference"),
                );
            }
            _ => {}
        }
    }

    let pages_ref = pages_ref.expect("Expected `Pages` in `Root`");
    let metadata_ref = metadata_ref.expect("Expected `Metadata` in `Root");
    println!("{:?}", pages_ref);
    println!("{:?}", metadata_ref);

    let pages = storage
        .resolve(pages_ref)
        .expect("Expected `Pages` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Pages` to be a dictionary");

    let metadata = storage
        .resolve(metadata_ref)
        .expect("Expected `Metadata` reference to be valid")
        .to_stream(&storage)
        .expect("Expected `Metadata` to be a dictionary");

    println!("metadata: {:?}", &metadata.info);
    println!(
        "```metadata\n{}\n```",
        std::str::from_utf8(&metadata.data).expect("Expect `Metadata` to be a valid utf-8 stream")
    );
    println!("pages: {:?}", pages);

    let mut pages_kids = None;
    for (key, value) in &pages {
        if key.as_str() == "Kids" {
            pages_kids = Some(
                value
                    .clone()
                    .to_array(&storage)
                    .expect("Expect `Pages`.`Kids` to be an array"),
            );
        }
    }

    let pages_kids = pages_kids.expect("Expect `Pages.Kids` to exist");
    for kid_ref in pages_kids {
        println!("{:?}", kid_ref);
        let kid = kid_ref
            .to_dictionary(&storage)
            .expect("Expect `Kids` entry to be a dictionary");

        println!("{:?}", kid);

        let mut contents_ref = None;
        let mut resources = None;
        for (key, value) in kid.iter() {
            match key.as_str() {
                "Contents" => {
                    contents_ref = Some(
                        value
                            .clone()
                            .to_reference()
                            .expect("Expect `Contents` to be a reference"),
                    );
                }
                "Resources" => {
                    resources = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expected `Metadata` to be a reference"),
                    );
                }
                _ => {}
            }
        }

        let resources = resources.expect("Expected `Resources` in `Page`");
        let contents_ref = contents_ref.expect("Expected `Contents` in `Page`");

        println!("resources: {:?}", resources);

        let mut ext_g_state = None;
        let mut font = None;
        for (key, value) in &resources {
            match key.as_str() {
                "Font" => {
                    font = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expect `Contents` to be a reference"),
                    );
                }
                "ExtGState" => {
                    ext_g_state = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expected `Metadata` to be a reference"),
                    );
                }
                _ => {}
            }
        }

        let ext_g_state = ext_g_state.expect("Expected `Page`.`ExtGState`");
        let font = font.expect("Expected `Page`.`Font`");

        println!("font: {}", font);

        for (key, value_ref) in &font {
            let value = value_ref
                .clone()
                .to_dictionary(&storage)
                .expect("Expect `Font` entry to be dictionary");
            println!("{}: {:#?}", key, value);

            let mut encoding = None;
            let mut to_unicode_ref = None;
            let mut char_procs = None;
            for (key, value) in &value {
                match key.as_str() {
                    "Encoding" => {
                        encoding = Some(
                            value
                                .clone()
                                .to_dictionary(&storage)
                                .expect("Expect `Encoding` to be a dictionary"),
                        );
                    }
                    "ToUnicode" => {
                        to_unicode_ref = Some(
                            value
                                .clone()
                                .to_reference()
                                .expect("Expected `ToUnicode` to be a reference"),
                        );
                    }
                    "CharProcs" => {
                        char_procs = Some(
                            value
                                .clone()
                                .to_dictionary(&storage)
                                .expect("Expected `CharProcs` to be a dictionary"),
                        );
                    }
                    _ => {}
                }
            }

            println!("to_unicode_ref: {:?}", to_unicode_ref);
            println!("char_procs: {:?}", char_procs);
            println!("encoding: {:?}", encoding);
        }

        println!("ext-g-state: {}", ext_g_state);

        println!("contents_ref: {:?}", contents_ref);

        let contents = storage
            .resolve(contents_ref)
            .expect("Expect `Contents` ref to be valid");
        let contents = contents
            .to_stream(&storage)
            .expect("Expected `Contents` to be stream");
        println!("contents.info{:?}", &contents.info);

        let content_stream =
            Stream::<()>::from_stream(contents, &storage).expect("Expect `Contents` to be valid");
        let decoded = content_stream
            .decode()
            .expect("Expect `Contents` decode to work");
        let decoded_text =
            std::str::from_utf8(&decoded).expect("Expect `Contents` to be valid utf-8");
        println!("decoded_text: {}", decoded_text);
    }
}

fn main() {
    println!("TODO");
}
