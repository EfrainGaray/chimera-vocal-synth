# Spanish

Speak and Sing read Spanish once you choose it: **ES**, beside EN and TR. The accent is a neutral
Latin American one (seseo and yeísmo). Spanish is written in Latin letters, so the lyrics router
can't tell it from English by script ([language-front-ends.md](language-front-ends.md)); English
stays the default.

`crates/chimera/src/phonetics/spanish.rs` reads the spelling by rule, with no dictionary, onto the
existing inventory (no new sounds). `chimera render spanish <out> "texto"` says it, and
`chimera syllables "letra" es` shows what Sing will sing.

## Writing it

- Type it as you'd write it, with its accents: á é í ó ú ü ñ, and ¿ ¡ if you like.
- An accent decides the stress and splits a vowel pair (país is pa-ís, día is dí-a).
- In Sing, a vowel is a note: words split where Spanish splits them (pe-rro, ciu-dad, ins-truc-ción).
  `-` alone holds the previous vowel onto the next note, and hyphens split a word your way.
- Numbers, times, ordinals and common abbreviations are read out in Speak: "1.500" is mil
  quinientos, "3,5" tres coma cinco, "10:30" diez y treinta, "2º" segundo, "Sr." señor, "EE. UU."
  Estados Unidos, "5 km" cinco kilómetros, "20 %" veinte por ciento.

## What it does

| | |
|---|---|
| c z s | /s/ (seseo); c before a o u, k and qu are /k/ |
| g j x | j, and g before e i, are /x/; gu before e i is /g/, gü /gw/; x is /ks/, /s/ word-initially, /x/ in México, Oaxaca, Texas, Ximena |
| ll y | /ʝ/ (yeísmo); y at the end of a word or alone is a vowel (hoy, muy, y) |
| h | silent; hue- is [w] (huevo), hie- is [ʝ] (hielo) |
| r | a trill [r] word-initially, after n l s and written rr; a tap [ɾ] elsewhere |
| b d g | [β ð ɣ] after a vowel or a consonant other than a nasal (and l, for d), across word boundaries in a phrase; stops after a pause or a nasal |
| n | takes the next consonant's place: [m] before p b m, [ɱ] before f, [ŋ] before k g x, [ɲ] before ch y ñ |
| s | [z] before a voiced consonant (mismo, desde) |
| p t k | unaspirated |
| i u | glides [j w] beside another vowel (bien, cuando, aire, auto), unless accented |
| Vowels | five, never reduced: the table's [i e o], [ɑ] for /a/ and the back [u̠] for /u/, the calibrated vowels nearest Spanish speakers' formants (Quilis & Esgueva 1983) |
| Stress | the written accent; else the penult when a word ends in a vowel, n or s, the last syllable otherwise; unstressed function words (el, la, de, que, mi…); -mente keeps its stem's stress as a secondary one |
| Rhythm | syllable-timed: unstressed vowels are lengthened against the English duration rules (×1.25), /a/ shortened to /e/'s length. Vowel nPVI 17 in the plan against English's 38 there (people: 30 against 57) |
| Intonation | statements fall; yes/no questions rise; a question starting with a question word (qué, dónde, cómo…) falls; a phrase ending in a comma, semicolon or colon rises a little (`continuation`: list items, a clause before the main one) |

In ES, Spanish is read before toned pinyin: Spanish writes acute accents where pinyin does, and
sí, tú, té, mí are real pinyin syllables. Hangul, Cyrillic, kana and Han characters still go to
their own front ends.

## How well it's understood

`tools/asr/spanish_wer.py` renders the 20 sentences of `tools/asr/sentences_es.tsv` in the male
and female voices and transcribes them with faster-whisper (greedy, Spanish; accents and
punctuation ignored):

| Model | Word error rate (male / female / all) |
|---|---|
| large-v3 | 4.7 / 4.0 / 4.3 % |
| small | 12.7 / 14.0 / 13.3 % |

The commonest errors: tr heard as g or t (tren as "gen"), "toda la tarde" as "todas las tardes",
the j of Jorge.

## Not yet

- A native speaker's ear. Only a recogniser has heard it.
- Synalepha (vowels joining across words: "la_amiga") and the shortened vowels around it.
- One stress per word part when a hyphen splits a word in Sing (ca-mi-no stresses each part).
- Regional accents (Spain's θ and ʎ, the Caribbean's aspirated s): one neutral accent.
- "1" before a noun reads "uno", not "un/una".
