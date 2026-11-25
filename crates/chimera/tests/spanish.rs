//! Spanish G2P: letters to sounds, allophones, stress, syllables, numbers, phrases and
//! intonation, and the lyrics front end.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::inventory::phoneme;
use chimera::phonetics::mandarin::MandarinLexicon;
use chimera::phonetics::planner::{PhraseInput, PlanOptions, is_vowel, plan_phrases};
use chimera::phonetics::spanish::{lyrics_to_syllables, to_phrases, word};
use chimera::phonetics::syllables::{Lang, lyrics_in};
use chimera::phonetics::{Intonation, Segment};

/// Segments as IPA, ˈ before a stressed vowel, ˌ before a secondary one, a space after each word.
fn shown(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| format!("{}{}{}", match s.stress { Some(1) => "ˈ", Some(2) => "ˌ", _ => "" }, s.ipa, if s.word_end == Some(true) { " " } else { "" }))
        .collect::<String>()
        .trim_end()
        .to_string()
}
fn es(text: &str) -> String {
    to_phrases(text).iter().map(|p| shown(&p.segments)).collect::<Vec<_>>().join(" | ")
}

/// Words as said alone (after a pause), with what a Latin American speaker says. Written from the
/// rules in Quilis (1993) and Hualde (2005), not from the front end's output.
#[rustfmt::skip]
const G2P: &[(&str, &str)] = &[
    // seseo: c (e, i), z and s are /s/; c, k, qu are /k/
    ("casa", "kˈɑsɑ"), ("cosa", "kˈosɑ"), ("cielo", "sjˈelo"), ("cena", "sˈenɑ"), ("zapato", "sɑpˈɑto"),
    ("zorro", "sˈoro"), ("caza", "kˈɑsɑ"), ("queso", "kˈeso"), ("quince", "kˈinse"), ("ceniza", "senˈisɑ"),
    ("kilo", "kˈilo"), ("cabeza", "kɑβˈesɑ"),
    // g: gu before e/i, gü, g before e/i and j are /x/
    ("guerra", "gˈerɑ"), ("guitarra", "gitˈɑrɑ"), ("pingüino", "piŋgwˈino"), ("cigüeña", "siɣwˈeɲɑ"),
    ("agua", "ˈɑɣwɑ"), ("gato", "gˈɑto"), ("gente", "xˈente"), ("girasol", "xiɾɑsˈol"), ("jamón", "xɑmˈon"),
    ("joven", "xˈoβen"), ("hijo", "ˈixo"), ("jirafa", "xiɾˈɑfɑ"), ("ojalá", "oxɑlˈɑ"), ("ángel", "ˈɑŋxel"),
    ("guapo", "gwˈɑpo"), ("antigüedad", "ɑntiɣweðˈɑð"), ("lingüística", "liŋgwˈistikɑ"),
    // h is silent; hue- is [w], hie- is [ʝ]
    ("hola", "ˈolɑ"), ("hombre", "ˈombɾe"), ("ahora", "ɑˈoɾɑ"), ("huevo", "wˈeβo"), ("hueso", "wˈeso"),
    ("hielo", "ʝˈelo"), ("hierba", "ʝˈeɾβɑ"), ("búho", "bˈu̠o"),
    // yeísmo: ll and consonantal y are /ʝ/; y at the end and alone is a vowel
    ("llave", "ʝˈɑβe"), ("lluvia", "ʝˈu̠βjɑ"), ("calle", "kˈɑʝe"), ("pollo", "pˈoʝo"), ("yo", "ʝˈo"),
    ("mayo", "mˈɑʝo"), ("ayer", "ɑʝˈeɾ"), ("hoy", "ˈoj"), ("muy", "mwˈi"), ("rey", "rˈej"), ("ley", "lˈej"),
    ("estoy", "estˈoj"), ("y", "i"), ("caballo", "kɑβˈɑʝo"), ("desayuno", "desɑʝˈu̠no"),
    // ñ
    ("niño", "nˈiɲo"), ("año", "ˈɑɲo"), ("España", "espˈɑɲɑ"), ("ñandú", "ɲɑndˈu̠"),
    // r: a trill at the start, after n l s and as rr; a tap elsewhere
    ("perro", "pˈero"), ("pero", "peɾo"), ("caro", "kˈɑɾo"), ("carro", "kˈɑro"), ("rosa", "rˈosɑ"),
    ("ratón", "rɑtˈon"), ("honra", "ˈonrɑ"), ("alrededor", "ɑlreðeðˈoɾ"), ("Israel", "izrɑˈel"),
    ("Enrique", "enrˈike"), ("arte", "ˈɑɾte"), ("mar", "mˈɑɾ"), ("ruido", "rwˈiðo"), ("río", "rˈio"),
    // clusters
    ("tres", "tɾˈes"), ("flor", "flˈoɾ"), ("blanco", "blˈɑŋko"), ("grande", "gɾˈɑnde"), ("dragón", "dɾɑɣˈon"),
    ("claro", "klˈɑɾo"), ("globo", "glˈoβo"), ("prisa", "pɾˈisɑ"), ("atlas", "ˈɑtlɑs"), ("hablar", "ɑβlˈɑɾ"),
    ("obstáculo", "oβstˈɑku̠lo"), ("submarino", "su̠βmɑɾˈino"),
    // x: /ks/; /s/ at the start; /x/ in México and its family
    ("examen", "eksˈɑmen"), ("taxi", "tˈɑksi"), ("tórax", "tˈoɾɑks"), ("xilófono", "silˈofono"),
    ("México", "mˈexiko"), ("Texas", "tˈexɑs"), ("Ximena", "ximˈenɑ"), ("Oaxaca", "oɑxˈɑkɑ"),
    ("psicología", "sikoloxˈiɑ"), ("acción", "ɑksjˈon"), ("lección", "leksjˈon"),
    // b and v are one sound; b d g are approximants except after a pause, a nasal (and d after l)
    ("vaca", "bˈɑkɑ"), ("vino", "bˈino"), ("uva", "ˈu̠βɑ"), ("lobo", "lˈoβo"), ("dedo", "dˈeðo"),
    ("nada", "nˈɑðɑ"), ("verdad", "beɾðˈɑð"), ("ciudad", "sju̠ðˈɑð"), ("Madrid", "mɑðɾˈið"), ("lago", "lˈɑɣo"),
    ("amigo", "ɑmˈiɣo"), ("algo", "ˈɑlɣo"), ("caldo", "kˈɑldo"), ("alba", "ˈɑlβɑ"), ("andar", "ɑndˈɑɾ"),
    ("ambos", "ˈɑmbos"), ("tango", "tˈɑŋgo"), ("club", "klˈu̠β"), ("Bogotá", "boɣotˈɑ"), ("Cuba", "kˈu̠βɑ"),
    // a nasal takes the next consonant's place
    ("un", "ˈu̠n"), ("enfermo", "eɱfˈeɾmo"), ("invierno", "imbjˈeɾno"), ("envidia", "embˈiðjɑ"),
    ("ancho", "ˈɑɲtʃo"), ("cónyuge", "kˈoɲʝu̠xe"), ("banco", "bˈɑŋko"), ("Colombia", "kolˈombjɑ"),
    // s is voiced before a voiced consonant
    ("mismo", "mˈizmo"), ("desde", "dˈezðe"), ("isla", "ˈizlɑ"),
    // ch
    ("chico", "tʃˈiko"), ("mucho", "mˈu̠tʃo"), ("noche", "nˈotʃe"), ("Chile", "tʃˈile"),
    // diphthongs: i and u by another vowel are glides
    ("fuego", "fwˈeɣo"), ("bueno", "bwˈeno"), ("cuando", "kwɑndo"), ("cuándo", "kwˈɑndo"),
    ("cuidado", "kwiðˈɑðo"), ("viuda", "bjˈu̠ðɑ"), ("aire", "ˈɑjɾe"), ("auto", "ˈɑwto"), ("aula", "ˈɑwlɑ"),
    ("reina", "rˈejnɑ"), ("Europa", "ewɾˈopɑ"), ("oigo", "ˈojɣo"), ("bien", "bjˈen"), ("tiene", "tjˈene"),
    ("piano", "pjˈɑno"), ("Paraguay", "pɑɾɑɣwˈɑj"), ("Uruguay", "u̠ɾu̠ɣwˈɑj"), ("buey", "bwˈej"),
    ("averiguáis", "ɑβeɾiɣwˈɑjs"), ("Guatemala", "gwɑtemˈɑlɑ"), ("Venezuela", "beneswˈelɑ"),
    ("Ecuador", "ekwɑðˈoɾ"), ("canción", "kɑnsjˈon"), ("miércoles", "mjˈeɾkoles"),
    ("dieciséis", "djesisˈejs"), ("veintiuno", "bejntjˈu̠no"),
    // hiatus: two strong vowels, or an accented i/u
    ("país", "pɑˈis"), ("maíz", "mɑˈis"), ("día", "dˈiɑ"), ("tío", "tˈio"), ("baúl", "bɑˈu̠l"),
    ("poeta", "poˈetɑ"), ("leer", "leˈeɾ"), ("caer", "kɑˈeɾ"), ("teatro", "teˈɑtɾo"), ("héroe", "ˈeɾoe"),
    ("océano", "osˈeɑno"),
    // stress: written accents, the default rule, unstressed function words, -mente
    ("corazón", "koɾɑsˈon"), ("café", "kɑfˈe"), ("sofá", "sofˈɑ"), ("árbol", "ˈɑɾβol"), ("lápiz", "lˈɑpis"),
    ("fácil", "fˈɑsil"), ("azúcar", "ɑsˈu̠kɑɾ"), ("música", "mˈu̠sikɑ"), ("pájaro", "pˈɑxɑɾo"),
    ("teléfono", "telˈefono"), ("cómetelo", "kˈometelo"), ("papel", "pɑpˈel"), ("reloj", "relˈox"),
    ("feliz", "felˈis"), ("comen", "kˈomen"), ("lunes", "lˈu̠nes"), ("virus", "bˈiɾu̠s"), ("Perú", "peɾˈu̠"),
    ("Argentina", "ɑɾxentˈinɑ"), ("el", "el"), ("los", "los"), ("de", "de"), ("qué", "kˈe"), ("tú", "tˈu̠"),
    ("él", "ˈel"), ("sí", "sˈi"), ("rápidamente", "rˌɑpiðɑmˈente"), ("felizmente", "felˌizmˈente"),
];

#[test]
fn the_g2p_table() {
    assert!(G2P.len() >= 150, "{} words", G2P.len());
    let wrong: Vec<String> = G2P.iter().filter_map(|(w, want)| {
        let got = shown(&word(w, true));
        (got != *want).then(|| format!("{w}: {got} (want {want})"))
    }).collect();
    assert!(wrong.is_empty(), "{} of {} wrong:\n{}", wrong.len(), G2P.len(), wrong.join("\n"));
}

#[test]
fn every_sound_is_in_the_inventory() {
    for (w, _) in G2P {
        for s in word(w, true) {
            assert!(is_vowel(&s.ipa, BodyMix::MALE) || phoneme(&s.ipa, BodyMix::MALE).is_some(), "{w}: {}", s.ipa);
        }
    }
}

/// Which syllable is stressed, counted from the end (1 aguda, 2 llana, 3 esdrújula, 4
/// sobresdrújula), and the syllables as the lyrics split them.
#[rustfmt::skip]
const STRESS: &[(&str, &str, usize)] = &[
    ("canción", "can-ción", 1), ("reloj", "re-loj", 1), ("papel", "pa-pel", 1), ("estoy", "es-toy", 1),
    ("café", "ca-fé", 1), ("ciudad", "ciu-dad", 1), ("cantad", "can-tad", 1), ("Israel", "Is-ra-el", 1),
    ("leer", "le-er", 1), ("país", "pa-ís", 1), ("Paraguay", "Pa-ra-guay", 1), ("averiguáis", "a-ve-ri-guáis", 1),
    ("hoy", "hoy", 1), ("bien", "bien", 1), ("ratón", "ra-tón", 1), ("corazón", "co-ra-zón", 1),
    ("casa", "ca-sa", 2), ("examen", "e-xa-men", 2), ("lunes", "lu-nes", 2), ("árbol", "ár-bol", 2),
    ("lápiz", "lá-piz", 2), ("fácil", "fá-cil", 2), ("azúcar", "a-zú-car", 2), ("día", "dí-a", 2),
    ("agua", "a-gua", 2), ("ruido", "rui-do", 2), ("tiene", "tie-ne", 2), ("cantan", "can-tan", 2),
    ("cantas", "can-tas", 2), ("virus", "vi-rus", 2), ("tórax", "tó-rax", 2), ("joven", "jo-ven", 2),
    ("crisis", "cri-sis", 2), ("poeta", "po-e-ta", 2), ("caracteres", "ca-rac-te-res", 2), ("perro", "pe-rro", 2),
    ("ahora", "a-ho-ra", 2), ("instrucción", "ins-truc-ción", 1), ("hombre", "hom-bre", 2), ("atlas", "a-tlas", 2),
    ("música", "mú-si-ca", 3), ("pájaro", "pá-ja-ro", 3), ("teléfono", "te-lé-fo-no", 3),
    ("miércoles", "miér-co-les", 3), ("régimen", "ré-gi-men", 3), ("regímenes", "re-gí-me-nes", 3),
    ("jóvenes", "jó-ve-nes", 3), ("héroe", "hé-ro-e", 3), ("océano", "o-cé-a-no", 3),
    ("cómetelo", "có-me-te-lo", 4), ("dígamelo", "dí-ga-me-lo", 4),
];

#[test]
fn stress_and_syllables() {
    let mut wrong = Vec::new();
    for (w, split, from_end) in STRESS {
        let syl = lyrics_to_syllables(w);
        let got_split = syl.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("-");
        let stressed = syl.iter().rposition(|s| s.nucleus.stress == Some(1)).map(|k| syl.len() - k);
        if got_split != *split || stressed != Some(*from_end) {
            wrong.push(format!("{w}: {got_split} stressed {stressed:?} (want {split}, {from_end})"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn allophones_cross_word_boundaries_in_a_phrase() {
    // b d g after a vowel in the word before are approximants; after a nasal, stops
    assert_eq!(es("la boca"), "lɑ βˈokɑ");
    assert_eq!(es("un beso"), "ˈu̠m bˈeso");
    assert_eq!(es("con ganas de llegar"), "koŋ gˈɑnɑz ðe ʝeɣˈɑɾ");
    assert_eq!(es("en Barcelona"), "em bɑɾselˈonɑ");
    // and the first word of a phrase starts with the stop
    assert_eq!(es("Dame el dedo."), "dˈɑme el dˈeðo");
}

#[test]
fn voiceless_stops_are_unaspirated() {
    let segs = word("tapa", true);
    assert!(segs.iter().filter(|s| s.ipa == "t" || s.ipa == "p").all(|s| s.aspirated == Some(false)));
}

#[test]
fn numbers_and_abbreviations_are_read_out() {
    assert_eq!(es("1"), es("uno"));
    assert_eq!(es("15"), es("quince"));
    assert_eq!(es("21"), es("veintiuno"));
    assert_eq!(es("100"), es("cien"));
    assert_eq!(es("101"), es("ciento uno"));
    assert_eq!(es("1518"), es("mil quinientos dieciocho"));
    assert_eq!(es("1.500"), es("mil quinientos"));
    assert_eq!(es("2024"), es("dos mil veinticuatro"));
    assert_eq!(es("21000"), es("veintiún mil"));
    assert_eq!(es("1000000"), es("un millón"));
    assert_eq!(es("3500000"), es("tres millones quinientos mil"));
    assert_eq!(es("3,5"), es("tres coma cinco"));
    assert_eq!(es("0,05"), es("cero coma cero cinco"));
    assert_eq!(es("20 %"), es("veinte por ciento"));
    assert_eq!(es("5 km"), es("cinco kilómetros"));
    assert_eq!(es("10:30"), es("diez y treinta"));
    assert_eq!(es("el 2º"), es("el segundo"));
    assert_eq!(es("la 1ª"), es("la primera"));
    assert_eq!(es("El Sr. García y la Dra. Pérez"), es("El señor García y la doctora Pérez"));
    assert_eq!(es("Ud. vive en EE. UU."), es("usted vive en Estados Unidos"));
    assert_eq!(es("Av. Juárez"), es("avenida Juárez"));
}

fn intonations(text: &str) -> Vec<Intonation> {
    to_phrases(text).iter().map(|p| p.intonation).collect()
}

#[test]
fn intonation_by_sentence_type() {
    use Intonation::{Continuation, Question, Statement};
    // a declarative falls; a yes/no question rises; a question-word question falls
    assert_eq!(intonations("Hoy llueve."), [Statement]);
    assert_eq!(intonations("¿Hoy llueve?"), [Question]);
    assert_eq!(intonations("¿Dónde vives?"), [Statement]);
    assert_eq!(intonations("¿Por qué no vienes?"), [Statement]);
    assert_eq!(intonations("Juan, ¿cuándo llegas?"), [Continuation, Statement]);
    assert_eq!(intonations("Juan, ¿vienes?"), [Continuation, Question]);
    // a list: each item but the last goes on
    assert_eq!(intonations("Compré peras, uvas, manzanas y naranjas."), [Continuation, Continuation, Statement]);
}

#[test]
fn a_continuation_rises_and_a_statement_falls() {
    // the last vowel's pitch target against the phrase's base
    let last_pitch = |text: &str| {
        let inputs: Vec<PhraseInput> = to_phrases(text).iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause), tune: p.tune }).collect();
        let plan = plan_phrases(&inputs[..1], &PlanOptions { body: BodyMix::MALE, f0: Some(100.0), ..Default::default() });
        plan.gestures.iter().filter(|g| g.label_str() == "pitch").map(|g| g.targets.get(chimera::control::score::ControlKey::F0).unwrap()).last().unwrap()
    };
    assert!(last_pitch("Compré peras, uvas.") > 115.0);
    assert!(last_pitch("Compré peras.") < 90.0);
    assert!(last_pitch("¿Compraste peras?") > 125.0);
}

/// Normalised pairwise variability of successive vowel durations (Grabe & Low 2002).
fn npvi(d: &[f64]) -> f64 {
    let n = d.len();
    100.0 * d.windows(2).map(|w| ((w[0] - w[1]) / ((w[0] + w[1]) / 2.0)).abs()).sum::<f64>() / (n - 1) as f64
}

#[test]
fn spanish_is_syllable_timed() {
    // the same kind of sentence in both languages: vowel durations vary about half as much in
    // Spanish (Grabe & Low 2002: Spanish 29.7, English 57.2)
    let vowel_durs = |phrases: Vec<chimera::phonetics::english::Phrase>| {
        let inputs: Vec<PhraseInput> = phrases.iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause), tune: p.tune }).collect();
        let plan = plan_phrases(&inputs, &PlanOptions { body: BodyMix::MALE, ..Default::default() });
        plan.segments.iter().filter(|s| s.kind == "vowel").map(|s| s.t1 - s.t0).collect::<Vec<_>>()
    };
    let es = npvi(&vowel_durs(to_phrases("La casa de mi madre tiene una ventana pequeña y un jardín con muchas flores amarillas.")));
    let en = npvi(&vowel_durs(chimera::phonetics::english::EnglishG2P::new().to_phrases("My mother's house has a little window and a garden with many yellow flowers.")));
    eprintln!("vowel nPVI: Spanish {es:.1}, English {en:.1}");
    assert!(es < 40.0 && es < 0.75 * en, "Spanish vowel nPVI {es:.1}, English {en:.1}");
    // stressed vowels are longer, but not twice as long
    let segs = to_phrases("pasaba").remove(0).segments;
    let inputs = [PhraseInput { segments: segs, intonation: Some(Intonation::Flat), pause: None, tune: None }];
    let plan = plan_phrases(&inputs, &PlanOptions { body: BodyMix::MALE, ..Default::default() });
    let v: Vec<_> = plan.segments.iter().filter(|s| s.kind == "vowel").collect();
    let ratio = (v[1].t1 - v[1].t0) / (v[0].t1 - v[0].t0);
    eprintln!("pasaba: stressed / unstressed vowel {ratio:.2}");
    assert!((1.05..1.7).contains(&ratio), "stressed/unstressed {ratio:.2}");
}

#[test]
fn lyrics_in_spanish() {
    let syl = lyrics_to_syllables("Ca-mi-no, guerra y pingüino");
    let text: Vec<&str> = syl.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(text, ["Ca", "mi", "no", "gue", "rra", "y", "pin", "güi", "no"]);
    assert!(syl[2].phrase_end);
    // the trill and the glide are in their syllables' onsets
    assert_eq!(syl[4].onset[0].ipa, "r");
    assert_eq!(syl[7].onset.iter().map(|s| s.ipa.as_str()).collect::<Vec<_>>(), ["g", "w"]);
    // a word after a vowel-final word starts with an approximant, after a nasal with a stop
    let d = |l: &str| lyrics_to_syllables(l)[1].onset[0].ipa.clone();
    assert_eq!(d("la boca"), "β");
    assert_eq!(d("un beso"), "b");
    // pinyin-looking accented words are Spanish in Spanish ("sí" is real pinyin, sí)
    let si = lyrics_in("sí, tú", &|_: &str, _: bool| Vec::<Segment>::new(), &MandarinLexicon::new(), Lang::Es);
    assert_eq!(si.iter().map(|s| s.nucleus.ipa.as_str()).collect::<Vec<_>>(), ["i", "u̠"]);
    // other scripts still go to their own front ends
    assert_eq!(lyrics_to_syllables("hola 사랑").len(), 4);
}

const STORY: &str = "En una noche de verano, una pequeña cucaracha salió a caminar por la cocina. Buscaba migas de pan, pero encontró un gato dormido. Con mucho cuidado, pasó por su lado sin hacer ruido.";

fn plan_es(text: &str, f0: f64) -> chimera::phonetics::Plan {
    let inputs: Vec<PhraseInput> = to_phrases(text).iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause), tune: p.tune }).collect();
    plan_phrases(&inputs, &PlanOptions { body: BodyMix::MALE, f0: Some(f0), ..Default::default() })
}

/// Mean vowel duration (ms) of the plan's stressed and unstressed vowels.
fn vowel_means(plan: &chimera::phonetics::Plan) -> (f64, f64) {
    let mean = |st: u8| {
        let v: Vec<f64> = plan.segments.iter().filter(|s| s.kind == "vowel" && s.stress == Some(st)).map(|s| 1000.0 * (s.t1 - s.t0)).collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    (mean(1), mean(0))
}

#[test]
fn vowels_are_as_short_as_read_spanish_and_stress_lengthens_them() {
    // read Spanish: stressed vowels 80-100 ms, unstressed 65-75, 1.2-1.5 times apart (Marín
    // Gálvez 1995); before this the rules gave 139 and 126 ms, a ratio of 1.1
    let (stressed, unstressed) = vowel_means(&plan_es(STORY, 110.0));
    let ratio = stressed / unstressed;
    eprintln!("story vowels: stressed {stressed:.0} ms, unstressed {unstressed:.0} ms, ratio {ratio:.2}");
    assert!((90.0..125.0).contains(&stressed), "stressed {stressed:.0} ms");
    assert!((65.0..100.0).contains(&unstressed), "unstressed {unstressed:.0} ms");
    assert!((1.2..1.5).contains(&ratio), "stressed/unstressed {ratio:.2}");
    // negative: the same segments without their Spanish lengths are the English rules' long vowels
    let mut phrases = to_phrases(STORY);
    for p in &mut phrases {
        for s in &mut p.segments {
            s.length = None;
        }
    }
    let inputs: Vec<PhraseInput> = phrases.iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause), tune: p.tune }).collect();
    let (s2, u2) = vowel_means(&plan_phrases(&inputs, &PlanOptions { body: BodyMix::MALE, f0: Some(110.0), ..Default::default() }));
    assert!(s2 > 125.0 && u2 > 100.0, "without lengths: {s2:.0} / {u2:.0} ms");
}

/// The pitch targets (Hz) of the gestures that start inside a planned vowel.
fn pitch_in(plan: &chimera::phonetics::Plan, seg: &chimera::phonetics::PlannedSegment) -> Vec<f64> {
    plan.gestures.iter().filter(|g| g.label_str() == "pitch" && g.t0 >= seg.t0 - 1e-9 && g.t0 < seg.t1 - 1e-9).filter_map(|g| g.targets.get(chimera::control::score::ControlKey::F0)).collect()
}

#[test]
fn prenuclear_accents_peak_on_the_syllable_after_the_stressed_one() {
    // "Mañana vamos a la playa": ña is a prenuclear accent. Spanish rises through it and peaks on
    // na (L+>H*: Face 2002); the planner's default (English) peaks on the stressed vowel itself.
    let f0 = 100.0;
    let plan = plan_es("Mañana vamos a la playa.", f0);
    let v: Vec<_> = plan.segments.iter().filter(|s| s.kind == "vowel").collect();
    assert_eq!(v[1].stress, Some(1), "{:?}", v.iter().map(|s| (&s.ipa, s.stress)).collect::<Vec<_>>());
    let on_stressed = pitch_in(&plan, v[1]);
    let on_next = pitch_in(&plan, v[2]);
    // the stressed vowel starts at the base and its rise reaches the peak the next vowel holds
    assert!(on_stressed[0] < f0 * 1.02, "ña starts at {:.1}", on_stressed[0]);
    let peak = *on_stressed.last().unwrap();
    assert!(peak > f0 * 1.12, "peak {peak:.1}");
    assert!((on_next[0] - peak).abs() < 1e-9, "na holds {:.1}, the peak is {peak:.1}", on_next[0]);
    // negative: with no tune, the peak is on the stressed vowel and the next one is lower
    let segs = to_phrases("Mañana vamos a la playa.").remove(0).segments;
    let plain = plan_phrases(&[PhraseInput { segments: segs, intonation: Some(Intonation::Statement), pause: None, tune: None }], &PlanOptions { body: BodyMix::MALE, f0: Some(f0), ..Default::default() });
    let pv: Vec<_> = plain.segments.iter().filter(|s| s.kind == "vowel").collect();
    let (a, b) = (pitch_in(&plain, pv[1])[0], pitch_in(&plain, pv[2])[0]);
    assert!(a > f0 * 1.1 && b < a, "untuned: ña {a:.1}, na {b:.1}");
}

#[test]
fn a_statement_declines_and_ends_under_its_base() {
    let f0 = 100.0;
    let plan = plan_es("Los niños de la escuela caminaban despacio por la vereda.", f0);
    let v: Vec<_> = plan.segments.iter().filter(|s| s.kind == "vowel").collect();
    // unstressed vowels sit on the base, which falls about 20 % over the phrase (the LibriVox
    // reader falls 3-6 semitones over a short phrase; the planner's default is 10 %)
    let first_unstressed = pitch_in(&plan, v[0])[0];
    let late_unstressed = pitch_in(&plan, v[v.len() - 3])[0];
    assert!(first_unstressed > late_unstressed * 1.12, "base {first_unstressed:.1} → {late_unstressed:.1}");
    // and the final low is under the base it ends at (about 4 semitones under the start)
    let last = *pitch_in(&plan, v[v.len() - 1]).last().unwrap();
    assert!(last < f0 * 0.78 && last < late_unstressed, "final low {last:.1}, base before it {late_unstressed:.1}");
    // negative: a question ends high
    let q = plan_es("¿Los niños caminaban despacio por la vereda?", f0);
    let qv: Vec<_> = q.segments.iter().filter(|s| s.kind == "vowel").collect();
    assert!(*pitch_in(&q, qv[qv.len() - 1]).last().unwrap() > f0 * 1.3);
}

#[test]
fn pauses_at_commas_and_full_stops() {
    let pauses: Vec<f64> = to_phrases("Hola, amigo. ¿Vienes? Chao").iter().map(|p| p.pause).collect();
    assert_eq!(pauses, [0.25, 0.55, 0.55, 0.55]);
    // negative: a phrase that goes on pauses less than a sentence's end
    assert!(pauses[0] < pauses[1]);
}

#[test]
fn intervocalic_approximants_are_brief() {
    // "la boca" and "el lago": [β ɣ] 45-60 ms between vowels; [ð] keeps the inventory's own 50 ms
    let plan = plan_es("La boca del lago abierto.", 110.0);
    let ms = |ipa: &str| plan.segments.iter().filter(|s| s.ipa == ipa).map(|s| 1000.0 * (s.t1 - s.t0)).fold(0.0, f64::max);
    eprintln!("β {:.0} ms, ɣ {:.0} ms, ð {:.0} ms", ms("β"), ms("ɣ"), ms("ð"));
    assert!(ms("β") <= 65.0 && ms("ɣ") <= 65.0, "β {:.0}, ɣ {:.0} ms", ms("β"), ms("ɣ"));
    assert!(ms("ð") > 35.0 && ms("ð") <= 65.0, "ð {:.0} ms", ms("ð"));
    // negative: the same segments without their length are the inventory's long fricatives
    let mut phrases = to_phrases("La boca del lago abierto.");
    for s in &mut phrases[0].segments {
        s.length = None;
    }
    let inputs = [PhraseInput { segments: phrases[0].segments.clone(), intonation: Some(Intonation::Statement), pause: None, tune: None }];
    let plain = plan_phrases(&inputs, &PlanOptions { body: BodyMix::MALE, f0: Some(110.0), ..Default::default() });
    let long = plain.segments.iter().filter(|s| s.ipa == "β").map(|s| 1000.0 * (s.t1 - s.t0)).fold(0.0, f64::max);
    assert!(long > 75.0, "untouched β {long:.0} ms");
}

#[test]
fn a_tap_after_an_alveolar_stop_strikes_from_the_stop() {
    // "otro": the tip is at the ridge after [t], so the tap's strike starts with its segment; after
    // a vowel ("oro") it starts 30 ms early, the tip's travel
    let strike_lead = |text: &str| {
        let plan = plan_es(text, 110.0);
        let tap = plan.segments.iter().find(|s| s.ipa == "ɾ").unwrap();
        let g = plan.gestures.iter().filter(|g| g.label_str() == "ɾ").map(|g| g.t0).fold(f64::MAX, f64::min);
        tap.t0 - g
    };
    assert!(strike_lead("otro") < 0.01, "otro: lead {:.3}", strike_lead("otro"));
    assert!(strike_lead("oro") > 0.025, "oro: lead {:.3}", strike_lead("oro"));
}
