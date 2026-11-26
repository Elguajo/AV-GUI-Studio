use figcraft_doc::{Document, NodeId, NodeKind, TextAlignH};

use crate::demo::{self as parts, INK};
use crate::model::{Color, Control, StateRole};
use crate::{ControlKind, DEFAULT_PROFILE, Insert, Kit, NEXT_ID_KEY, assign, demo, insert, panel, panel_frame, project};

/// The editor's component sync, which builds the instances of masters.
const SYNC: crate::SyncInstance = figcraft_engine::components::sync_instance;

fn add(doc: &mut Document, kind: ControlKind) -> crate::Assigned {
    insert(doc, kind, Insert::default(), SYNC).unwrap()
}

#[test]
fn a_new_panel_takes_every_control_with_presets_and_exports() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let errors = project(&doc).unwrap_err();
    assert_eq!(errors[0].message, "A screen needs 1–64 elements.");

    let card =
        insert(&mut doc, ControlKind::Shape, Insert { name: Some("Card".into()), x: Some(40), y: Some(60), ..Insert::default() }, SYNC).unwrap();
    let title =
        insert(&mut doc, ControlKind::Label, Insert { text: Some("Huddle Room".into()), x: Some(60), y: Some(80), ..Insert::default() }, SYNC)
            .unwrap();
    let mute =
        insert(&mut doc, ControlKind::Button, Insert { name: Some("Mute".into()), text: Some("Mute".into()), ..Insert::default() }, SYNC).unwrap();
    let volume = add(&mut doc, ControlKind::Slider);

    assert_eq!([card.object_id, title.object_id, mute.object_id, volume.object_id], [101, 102, 103, 104]);
    assert_eq!([card.control_id, title.control_id, mute.control_id, volume.control_id], [None, None, Some(103), Some(104)]);
    assert_eq!(mute.key, "studio.element-103");

    let screen = project(&doc).unwrap();
    assert_eq!((screen.name.as_str(), screen.pages[0].identity.object_id), ("Huddle Room", 100));
    let bounds: Vec<_> = screen.pages[0].elements.iter().map(|e| (e.name.as_str(), e.bounds)).collect();
    assert_eq!(bounds, [("Card", [40, 60, 256, 172]), ("Label", [60, 80, 216, 44]), ("Mute", [545, 364, 190, 72]), ("Slider", [504, 376, 272, 48])]);
    let Control::Label { text, size, .. } = &screen.pages[0].elements[1].control else { panic!("a label") };
    assert_eq!((text.as_str(), *size), ("Huddle Room", 22));
    let Control::Button { states, size } = &screen.pages[0].elements[2].control else { panic!("a button") };
    assert_eq!(states.each_ref().map(|s| s.text.as_str()), ["Mute"; 3]);
    assert_eq!(*size, 21);
    let Control::Slider { min, max, .. } = screen.pages[0].elements[3].control else { panic!("a slider") };
    assert_eq!((min, max), (0, 100));
    crate::package(&screen).unwrap();
}

#[test]
fn ids_of_deleted_controls_are_never_handed_out_again() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    add(&mut doc, ControlKind::Label);
    let gone = add(&mut doc, ControlKind::Shape);
    doc.remove(gone.node).unwrap();
    assert_eq!(add(&mut doc, ControlKind::Shape).object_id, gone.object_id + 1);
    let panel = panel_frame(&doc).unwrap();
    assert_eq!(doc.node(panel).unwrap().plugin_data[NEXT_ID_KEY], "104");

    // A screen without a counter (the demo, an older file) continues above its highest ID.
    let mut demo = demo().unwrap();
    assert_eq!(add(&mut demo, ControlKind::Label).object_id, 8116);
    assert_eq!(project(&demo).unwrap().pages[0].elements.len(), 16);
}

#[test]
fn assigning_keeps_a_controls_ids_and_renews_a_copys() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let frame = panel_frame(&doc).unwrap();
    let title = insert(&mut doc, ControlKind::Label, Insert { x: Some(60), y: Some(80), ..Insert::default() }, SYNC).unwrap();
    assert_eq!(assign(&mut doc, title.node, ControlKind::Label).unwrap(), title);

    let copy = doc.duplicate_into(title.node, frame, None).unwrap();
    assert!(project(&doc).unwrap_err().iter().any(|e| e.node_id == copy && e.message.contains("Copied Extron assignment")));
    let renewed = assign(&mut doc, copy, ControlKind::Label).unwrap();
    assert_eq!((renewed.object_id, renewed.key.as_str()), (102, "studio.element-102"));

    // A plain text layer drawn in the panel becomes a label.
    let plain = parts::text_layer(&mut doc, "Hint", [60, 200, 300, 40], "Hint".into(), INK, 18, TextAlignH::Left);
    let plain = parts::insert(&mut doc, plain, frame).unwrap();
    assert_eq!(assign(&mut doc, plain, ControlKind::Label).unwrap().object_id, 103);
    assert_eq!(project(&doc).unwrap().pages[0].elements.len(), 3);
}

#[test]
fn a_hand_made_variant_set_becomes_a_button_with_ordered_state_roles() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let frame = panel_frame(&doc).unwrap();
    let root = doc.current_page.unwrap();
    let set = parts::node(&mut doc, NodeKind::ComponentSet { description: String::new() }, "Power", [1400, 0, 640, 100]);
    let set = parts::insert(&mut doc, set, root).unwrap();
    let fills = [Color([40, 40, 40]), Color([0, 160, 90]), Color([0, 110, 60])];
    let mut first = None;
    for (index, fill) in fills.into_iter().enumerate() {
        let kind = NodeKind::Component { description: String::new(), props: Default::default(), prop_order: Vec::new() };
        let mut variant = parts::node(&mut doc, kind, &format!("State={index}"), [12 + index as i32 * 212, 12, 200, 60]);
        parts::fill(&mut variant, fill);
        let variant = parts::insert(&mut doc, variant, set).unwrap();
        let caption = parts::text_layer(&mut doc, "Caption", [0, 0, 200, 60], "Power".into(), INK, 20, TextAlignH::Center);
        parts::insert(&mut doc, caption, variant).unwrap();
        first.get_or_insert(variant);
    }
    let component = first.unwrap();
    let mut instance = parts::node(&mut doc, NodeKind::Instance { component, props: Default::default() }, "Power", [60, 300, 200, 60]);
    instance.source = Some(component);
    parts::fill(&mut instance, fills[0]);
    let instance = parts::insert(&mut doc, instance, frame).unwrap();

    let power = assign(&mut doc, instance, ControlKind::Button).unwrap();
    assert_eq!((power.object_id, power.control_id), (101, Some(101)));
    let screen = project(&doc).unwrap();
    let Control::Button { states, .. } = &screen.pages[0].elements[0].control else { panic!("a button") };
    assert_eq!(states.each_ref().map(|s| s.name.as_str()), StateRole::ALL.map(StateRole::name));
    assert_eq!(states.each_ref().map(|s| s.fill), fills);
}

#[test]
fn refuses_layers_outside_the_panel_wrong_layer_types_and_pages_without_a_panel() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let root = doc.current_page.unwrap();
    let loose = parts::text_layer(&mut doc, "Loose", [1400, 0, 100, 40], "Loose".into(), INK, 18, TextAlignH::Left);
    let loose: NodeId = parts::insert(&mut doc, loose, root).unwrap();
    assert_eq!(assign(&mut doc, loose, ControlKind::Label).unwrap_err(), "\"Loose\" must be directly inside an Extron panel.");
    let card = add(&mut doc, ControlKind::Shape);
    assert_eq!(assign(&mut doc, card.node, ControlKind::Label).unwrap_err(), "A label must be a text layer.");
    assert_eq!(assign(&mut doc, card.node, ControlKind::Button).unwrap_err(), "A button must be an instance of a button variant set.");
    assert_eq!("dial".parse::<ControlKind>().unwrap_err(), "Unknown control kind \"dial\". Use button, slider, label or shape.");

    let mut plain = Document::new("Sketch");
    assert_eq!(add_err(&mut plain), "This page has no Extron panel. Start with a new touch panel project.");
    assert_eq!(panel(" ", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap_err(), "Project name needs 1–256 characters.");
}

fn add_err(doc: &mut Document) -> String {
    insert(doc, ControlKind::Label, Insert::default(), SYNC).unwrap_err()
}

fn screen_json(doc: &Document) -> serde_json::Value {
    let bytes = crate::package(&project(doc).unwrap()).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    serde_json::from_reader(archive.by_name("screen.json").unwrap()).unwrap()
}

#[test]
fn added_pages_share_the_panel_and_one_id_counter_and_export_as_a_project() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    add(&mut doc, ControlKind::Label);
    let home = panel_frame(&doc).unwrap();
    let audio = crate::add_page(&mut doc, crate::NewPage::default()).unwrap();
    let node = doc.node(audio).unwrap();
    assert_eq!((node.name.as_str(), node.width, node.height), ("Page 2", 1280.0, 800.0));
    // Right of the Home page, which is the only layer on the canvas yet, and level with it.
    assert_eq!(node.transform.as_coeffs()[4..], [1280.0 + 160.0, 0.0]);
    assert_eq!(crate::panels(&doc), vec![home, audio]);
    assert_eq!(panel_frame(&doc).unwrap(), home);

    // A control goes to the page asked for, with an ID above the new page's own.
    let mute = insert(&mut doc, ControlKind::Button, Insert { panel: Some(audio), text: Some("Mute".into()), ..Insert::default() }, SYNC).unwrap();
    assert_eq!(mute.object_id, 103);
    assert_eq!(crate::panel_of(&doc, mute.node), Some(audio));
    assert_eq!(doc.node(home).unwrap().plugin_data[NEXT_ID_KEY], "104");
    assert_eq!(doc.node(audio).unwrap().plugin_data[NEXT_ID_KEY], "104");

    let exported = screen_json(&doc);
    assert!(exported.get("page").is_none());
    let pages = exported["pages"].as_array().unwrap();
    let summary: Vec<_> =
        pages.iter().map(|p| (p["name"].as_str().unwrap(), p["objectId"].as_u64().unwrap(), p["shownBy"].as_str().unwrap())).collect();
    assert_eq!(summary, [("Home", 100, "start"), ("Page 2", 102, "program")]);
    assert_eq!(pages[1]["elements"][0]["controlId"], 103);
    assert_eq!((exported["popups"].as_array().unwrap().len(), exported["groups"].as_array().unwrap().len()), (0, 0));

    // Removing the page keeps its IDs retired; a page name must be unique.
    doc.remove(audio).unwrap();
    assert_eq!(add(&mut doc, ControlKind::Shape).object_id, 104);
    // The button master stays on the canvas for the next button and doesn't block export.
    assert!(doc.nodes().any(|n| matches!(n.kind, NodeKind::ComponentSet { .. }) && n.plugin_data.contains_key(crate::MASTER_KEY)));
    assert!(screen_json(&doc).get("page").is_some());
    let named = crate::add_page(&mut doc, crate::NewPage { name: Some("Audio".into()), at: Some([0, 1000]) }).unwrap();
    assert_eq!(doc.node(named).unwrap().transform.as_coeffs()[4..], [0.0, 1000.0]);
    assert_eq!(
        crate::add_page(&mut doc, crate::NewPage { name: Some("Audio".into()), at: None }).unwrap_err(),
        "Page name \"Audio\" is already used by another page."
    );
    let errors = project(&doc).unwrap_err();
    assert_eq!((errors[0].node_id, errors[0].message.as_str()), (named, "A screen needs 1–64 elements."));
}

#[test]
fn a_300m_project_uses_its_portrait_resolution_everywhere() {
    let portrait = crate::profile("tlp-pro-300m-portrait").unwrap();
    let mut doc = panel("Lectern", portrait, &Kit::default_kit().unwrap()).unwrap();
    let home = panel_frame(&doc).unwrap();
    assert_eq!((doc.node(home).unwrap().width, doc.node(home).unwrap().height), (320.0, 480.0));
    let mute = insert(&mut doc, ControlKind::Button, Insert { text: Some("Mute".into()), ..Insert::default() }, SYNC).unwrap();
    let volume = add(&mut doc, ControlKind::Slider);
    let page = crate::add_page(&mut doc, crate::NewPage::default()).unwrap();
    assert_eq!(doc.node(page).unwrap().width, 320.0);
    insert(&mut doc, ControlKind::Shape, Insert { panel: Some(page), ..Insert::default() }, SYNC).unwrap();

    let screen = project(&doc).unwrap();
    assert_eq!(screen.profile.id, "tlp-pro-300m-portrait");
    let bounds: Vec<_> = screen.pages[0].elements.iter().map(|e| e.bounds).collect();
    assert_eq!(bounds, [[65, 204, 190, 72], [24, 216, 272, 48]]);
    assert_eq!((mute.control_id, volume.control_id), (Some(101), Some(102)));
    let bytes = crate::package(&screen).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let manifest: serde_json::Value = serde_json::from_reader(archive.by_name("manifest.json").unwrap()).unwrap();
    assert_eq!(manifest["profile"], "tlp-pro-300m-portrait");

    // A layer past the portrait bounds names them; a page of another panel model is refused.
    doc.node_mut(volume.node).unwrap().transform = figcraft_geom::Affine::translate((100.0, 216.0));
    let errors = project(&doc).unwrap_err();
    assert_eq!(errors[0].message, "Element must fit inside the 320 × 480 screen.");
}

fn built_in(id: &str) -> Kit {
    Kit::built_in().into_iter().find(|kit| kit.id == id).unwrap()
}

#[test]
fn the_built_in_kits_are_valid_and_aurora_keeps_the_original_presets() {
    let ids: Vec<_> = Kit::built_in().into_iter().map(|kit| kit.id).collect();
    assert_eq!(ids, ["aurora", "daylight", "graphite"]);
    assert_eq!(Kit::default_kit().unwrap().id, crate::DEFAULT_KIT);
    // A project without a stored kit (the demo, an older file) inserts in Aurora's look, which is
    // the demo's: a 21 pt Mute caption on #213D45 with mint feedback.
    let mut doc = demo().unwrap();
    assert_eq!(crate::project_kit(&doc).unwrap(), Kit::default_kit().unwrap());
    let mute = insert(&mut doc, ControlKind::Button, Insert { text: Some("Mute".into()), ..Insert::default() }, SYNC).unwrap();
    let screen = project(&doc).unwrap();
    let item = screen.pages[0].elements.iter().find(|e| e.node == mute.node).unwrap();
    let Control::Button { states, size } = &item.control else { panic!("a button") };
    assert_eq!((states.each_ref().map(|s| s.fill), *size), ([Color([33, 61, 69]), Color([122, 234, 188]), Color([82, 175, 145])], 21));
}

#[test]
fn a_projects_kit_styles_its_pages_controls_and_blocks() {
    let daylight = built_in("daylight");
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &daylight).unwrap();
    let home = panel_frame(&doc).unwrap();
    assert_eq!(crate::project_kit(&doc).unwrap().id, "daylight");
    let page = crate::add_page(&mut doc, crate::NewPage::default()).unwrap();
    assert_eq!(doc.node(page).unwrap().plugin_data[crate::KIT_KEY], doc.node(home).unwrap().plugin_data[crate::KIT_KEY]);

    let placed = crate::insert_block(&mut doc, &daylight, "Source selection", Some(page), None, SYNC).unwrap();
    assert_eq!(placed.len(), 6);
    assert_eq!(placed.iter().map(|a| a.object_id).collect::<Vec<_>>(), [102, 103, 104, 105, 106, 107]);
    let card = add(&mut doc, ControlKind::Shape); // on the start page
    let screen = project(&doc).unwrap();
    assert_eq!(screen.pages[0].background, Color([0xF2, 0xF4, 0xF7]));
    let Control::Shape { fill, radius } = screen.pages[0].elements.iter().find(|e| e.node == card.node).unwrap().control else { panic!("a shape") };
    assert_eq!((fill, radius), (Color([0xE4, 0xE7, 0xEC]), 12));
    // The block is centred on its page: 704 × 348 on 1280 × 800.
    let blocks: Vec<_> = screen.pages[1].elements.iter().map(|e| (e.name.as_str(), e.bounds)).collect();
    assert_eq!(blocks[0], ("Source card", [288, 226, 704, 348]));
    assert_eq!(blocks[3], ("HDMI", [319, 384, 201, 82]));
    let Control::Label { color, size, .. } = &screen.pages[1].elements[2].control else { panic!("a hint") };
    assert_eq!((*color, *size), (Color([0x66, 0x70, 0x85]), 17));
    let Control::Button { states, .. } = &screen.pages[1].elements[3].control else { panic!("a button") };
    assert_eq!(states[1].fill, Color([0x2E, 0x90, 0xFA]));
    assert_eq!(states[0].text, "HDMI");
    crate::package(&screen).unwrap();

    // Another kit for the controls placed from now on; those placed keep their look.
    let graphite = built_in("graphite");
    crate::set_kit(&mut doc, &graphite).unwrap();
    let later = add(&mut doc, ControlKind::Shape);
    let screen = project(&doc).unwrap();
    let Control::Shape { fill, .. } = screen.pages[0].elements.iter().find(|e| e.node == later.node).unwrap().control else { panic!("a shape") };
    assert_eq!(fill, Color([0x1E, 0x1E, 0x1E]));
    assert_eq!(screen.pages[1].elements.len(), 6);
}

#[test]
fn a_block_has_to_fit_the_page() {
    let portrait = crate::profile("tlp-pro-300m-portrait").unwrap();
    let aurora = Kit::default_kit().unwrap();
    let mut doc = panel("Lectern", portrait, &aurora).unwrap();
    let error = crate::insert_block(&mut doc, &aurora, "Source selection", None, None, SYNC).unwrap_err();
    assert_eq!(error, "\"Source selection\" is 704 × 348 and does not fit the 320 × 480 page.");
    assert!(crate::insert_block(&mut doc, &aurora, "Nothing", None, None, SYNC).unwrap_err().contains("has no block \"Nothing\""));
    let volume = crate::insert_block(&mut doc, &aurora, "Volume", None, Some([12, 300]), SYNC).unwrap();
    assert_eq!(volume.len(), 3);
    let screen = project(&doc).unwrap();
    assert_eq!(screen.pages[0].elements[1].bounds, [12, 344, 296, 45]);
}

#[test]
fn a_kit_file_is_checked_before_it_is_used() {
    let raw = serde_json::to_value(Kit::default_kit().unwrap()).unwrap();
    let parse = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut value = raw.clone();
        edit(&mut value);
        Kit::parse(&serde_json::to_vec(&value).unwrap())
    };
    assert_eq!(parse(&|_| {}).unwrap(), Kit::default_kit().unwrap());
    assert!(parse(&|v| v["schema"] = "gui-studio-kit/9".into()).unwrap_err().contains("Unsupported UI kit schema"));
    assert!(parse(&|v| v["id"] = "My Kit".into()).unwrap_err().contains("lowercase letters"));
    assert_eq!(parse(&|v| v["background"] = "teal".into()).unwrap_err(), "\"teal\" is not a #RRGGBB colour.");
    assert!(parse(&|v| v["button"]["active"] = v["button"]["normal"].clone()).unwrap_err().contains("need different colours"));
    assert!(parse(&|v| v["label"]["size"] = 2.into()).unwrap_err().contains("4–200 pt"));
    assert!(parse(&|v| v["shadow"] = true.into()).unwrap_err().starts_with("This is not a UI kit file"));
    let outside = parse(&|v| v["blocks"][0]["items"][0]["rect"] = serde_json::json!([0, 0, 900, 36])).unwrap_err();
    assert_eq!(outside, "Block \"Header\": \"Overline\" must lie inside the 860 × 120 block.");
    let kind = parse(&|v| v["blocks"][0]["items"][0]["kind"] = "dial".into()).unwrap_err();
    assert!(kind.contains("\"dial\" is not a control kind"), "{kind}");
    assert!(Kit::parse(&vec![b' '; 300 * 1024]).unwrap_err().contains("256 KiB"));
}

/// The master marked `tag` (such as `button:aurora`) on the canvas page.
fn master(doc: &Document, tag: &str) -> NodeId {
    let found: Vec<_> = doc.nodes().filter(|n| n.plugin_data.get(crate::MASTER_KEY).map(String::as_str) == Some(tag)).map(|n| n.id).collect();
    assert_eq!(found.len(), 1, "one {tag} master");
    found[0]
}

fn button_states(doc: &Document, node: NodeId) -> [crate::model::ButtonState; 3] {
    let screen = project(doc).unwrap();
    let item = screen.pages.iter().flat_map(|p| &p.elements).find(|e| e.node == node).unwrap();
    let Control::Button { states, .. } = &item.control else { panic!("a button") };
    states.clone()
}

#[test]
fn buttons_and_sliders_are_instances_of_one_master_that_restyles_them_all() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let button = |doc: &mut Document, text: &str, x: i32, size: Option<[i32; 2]>| {
        insert(
            doc,
            ControlKind::Button,
            Insert { name: Some(text.into()), text: Some(text.into()), x: Some(x), y: Some(100), size, ..Insert::default() },
            SYNC,
        )
        .unwrap()
    };
    let hdmi = button(&mut doc, "HDMI", 40, None);
    let usb = button(&mut doc, "USB-C", 260, None);
    let mute = button(&mut doc, "Mute", 480, Some([363, 62]));
    let plain = button(&mut doc, "Button", 880, None);
    let volume = add(&mut doc, ControlKind::Slider);
    let wide = insert(&mut doc, ControlKind::Slider, Insert { y: Some(600), size: Some([500, 48]), ..Insert::default() }, SYNC).unwrap();

    // One master per kind beside the panel; every control is an instance of it.
    let set = master(&doc, "button:aurora");
    let slider = master(&doc, "slider:aurora");
    let normal = doc.node(set).unwrap().children[0];
    for (item, main) in [(&hdmi, normal), (&usb, normal), (&mute, normal), (&plain, normal), (&volume, slider), (&wide, slider)] {
        let NodeKind::Instance { component, .. } = doc.node(item.node).unwrap().kind else { panic!("an instance") };
        assert_eq!(component, main);
    }
    assert!(doc.node(set).unwrap().transform.as_coeffs()[4] >= 1280.0, "the master sits beside the panel");
    // The caption is the instance's own and names every state; the master's needs no override.
    assert_eq!(button_states(&doc, mute.node).each_ref().map(|s| s.text.as_str()), ["Mute"; 3]);
    assert!(doc.node(doc.node(plain.node).unwrap().children[0]).unwrap().overrides.is_empty());
    // A size of its own stretches the caption with it.
    let mute_layer = doc.node(mute.node).unwrap();
    assert_eq!((mute_layer.width, mute_layer.height, mute_layer.overrides.contains("size")), (363.0, 62.0, true));
    assert_eq!(doc.node(mute_layer.children[0]).unwrap().width, 363.0);
    let ids = [hdmi.object_id, usb.object_id, mute.object_id, plain.object_id, volume.object_id, wide.object_id];

    // Restyle the masters: every button and slider follows, with its caption, size and IDs.
    parts::fill(doc.node_mut(normal).unwrap(), Color([200, 40, 40]));
    let thumb = doc.node(slider).unwrap().children[2];
    parts::fill(doc.node_mut(thumb).unwrap(), Color([250, 250, 0]));
    for item in [&hdmi, &usb, &mute, &plain, &volume, &wide] {
        SYNC(&mut doc, item.node);
    }
    for (item, text) in [(&hdmi, "HDMI"), (&usb, "USB-C"), (&mute, "Mute"), (&plain, "Button")] {
        let states = button_states(&doc, item.node);
        assert_eq!((states[0].fill, states[0].text.as_str(), states[1].text.as_str()), (Color([200, 40, 40]), text, text));
    }
    assert_eq!(doc.node(mute.node).unwrap().width, 363.0);
    let screen = project(&doc).unwrap();
    for item in [&volume, &wide] {
        let element = screen.pages[0].elements.iter().find(|e| e.node == item.node).unwrap();
        let Control::Slider { thumb, .. } = element.control else { panic!("a slider") };
        assert_eq!(thumb, Color([250, 250, 0]));
    }
    let after: Vec<_> = screen.pages[0].elements.iter().map(|e| e.identity.object_id).collect();
    assert_eq!(after, ids);
}

#[test]
fn a_problem_in_a_master_is_reported_once_at_the_master() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let buttons: Vec<_> = [40, 260, 480]
        .into_iter()
        .map(|x| insert(&mut doc, ControlKind::Button, Insert { x: Some(x), ..Insert::default() }, SYNC).unwrap().node)
        .collect();
    let set = master(&doc, "button:aurora");
    let pressed = doc.node(set).unwrap().children[2];
    doc.node_mut(pressed).unwrap().opacity = 0.5;
    let errors = project(&doc).unwrap_err();
    let at_master: Vec<_> = errors.iter().filter(|e| e.node_id == pressed).collect();
    assert_eq!(at_master.len(), 1, "{errors:?}");
    assert_eq!(at_master[0].layer, "State=Pressed");
    assert!(errors.iter().all(|e| !buttons.contains(&e.node_id)), "{errors:?}");
    // The other kit's controls get their own master.
    insert(&mut doc, ControlKind::Button, Insert { kit: Some(built_in("graphite")), ..Insert::default() }, SYNC).unwrap();
    master(&doc, "button:graphite");
}

#[test]
fn a_button_refuses_a_look_changed_on_the_instance() {
    let mut doc = panel("Huddle Room", DEFAULT_PROFILE, &Kit::default_kit().unwrap()).unwrap();
    let power = insert(&mut doc, ControlKind::Button, Insert { text: Some("Power".into()), ..Insert::default() }, SYNC).unwrap();
    let caption = doc.node(power.node).unwrap().children[0];
    let layer = doc.node_mut(caption).unwrap();
    parts::fill(layer, INK);
    layer.overrides.insert("fills".into());
    let errors = project(&doc).unwrap_err();
    assert_eq!(
        (errors[0].node_id, errors[0].message.as_str()),
        (power.node, "Only the caption, size and position can change on a button. Edit its look on the master's state variants.")
    );
}
