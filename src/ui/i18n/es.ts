/**
 * Spanish for every text the UI can write. Keys are the exact English strings used in the code;
 * anything not listed stays English (by design, so the app never shows a hole).
 *
 * Deliberately not translated: IPA symbols and chart consonants (phonetics), note names, units,
 * body-part Latin names, demo song titles, and the example sentences (the engine speaks English).
 */

// ---------------------------------------------------------------- top bar, shared screens
export const ES: Record<string, string> = {
  Language: "Idioma",
  "A voice grown from a simulated airway": "Una voz cultivada de una vía aérea simulada",
  Mode: "Modo",
  Play: "Tocar",
  Speak: "Hablar",
  Sing: "Cantar",
  Bestiary: "Bestiario",
  Body: "Cuerpo",
  Male: "Masculino",
  Female: "Femenino",
  Child: "Niño",
  Volume: "Volumen",
  "Share of the audio thread's time budget used by the simulation":
    "Parte del presupuesto de tiempo del hilo de audio que usa la simulación",
  "Engine load": "Carga",
  "Starting…": "Iniciando…",
  "Start audio": "Iniciar audio",
  "Source code on GitHub": "Código fuente en GitHub",
  Breathe: "Respirar",
  "Press again to retry.": "Pulsa para reintentar.",

  "Sound pressure": "Presión sonora",
  Airflow: "Flujo de aire",
  Labels: "Etiquetas",
  Measurements: "Mediciones",
  Grid: "Cuadrícula",

  Creature: "Criatura",
  "Vocal tract": "Tracto vocal",
  "Airway / live": "Vía aérea / en vivo",
  midsagittal: "sagital media",
  "Live creature airway": "Vía aérea de la criatura, en vivo",
  "Live vocal tract": "Tracto vocal en vivo",

  Tube: "Tubo",
  "Glottis to lips": "De la glotis a los labios",
  "Vocal folds": "Cuerdas vocales",
  "Slowed 100×": "A cámara lenta, 100×",
  Spectrum: "Espectro",
  Resonances: "Resonancias",
  Vowels: "Vocales",
  Voice: "Voz",
  Keys: "Teclas",

  // ---------------------------------------------------------------- start prompts
  "Every sound here is produced by simulating air moving through a throat, mouth and nose.":
    "Cada sonido aquí se produce simulando aire que se mueve por garganta, boca y nariz.",
  "Each animal's sound comes from a simulation of its own sound source and airway.":
    "El sonido de cada animal viene de una simulación de su propia fuente sonora y su vía aérea.",
  "Every drum here is a mouth sound: lips, tongue and larynx pushing, sealing and sucking air.":
    "Cada tambor aquí es un sonido de boca: labios, lengua y laringe empujando, sellando y succionando aire.",
  "Start and hear a call": "Inicia y escucha un llamado",
  "Start the beat": "Inicia el ritmo",
  "Start the voice": "Inicia la voz",
  "Start and hear a song": "Inicia y escucha una canción",
  "Breathe in and start": "Toma aire y empieza",
  "Then play its calls, or hold the keys to sustain it at a pitch.":
    "Luego toca sus llamados, o mantén las teclas para sostenerlo a una altura.",
  "Then edit the pattern, tap the pads, or hum a bass note on the keys.":
    "Luego edita el patrón, golpea las almohadillas o tararea una nota de bajo en las teclas.",
  "Then type something and press Speak, or tap a sound in the chart.":
    "Luego escribe algo y pulsa Hablar, o toca un sonido en la tabla.",
  "Then play the keys: each note sings the next syllable of the lyrics.":
    "Luego toca las teclas: cada nota canta la siguiente sílaba de la letra.",
  "Then drag the tongue, lips or jaw, or play the keys.":
    "Luego arrastra la lengua, los labios o la mandíbula, o toca las teclas.",
  "Audio could not start": "No se pudo iniciar el audio",
  "Reload the page and try again.": "Recarga la página y prueba de nuevo.",

  // ---------------------------------------------------------------- boot
  "The voice engine could not load": "No se pudo cargar el motor de voz",
  "Try again": "Probar de nuevo",
  "3D needs WebGL, which this browser has turned off. Untick 3D for the flat view.":
    "El 3D necesita WebGL, que este navegador tiene desactivado. Desmarca 3D para la vista plana.",

  // ---------------------------------------------------------------- body
  "Adult male": "Hombre adulto",
  "Adult female": "Mujer adulta",
  "Body between male, female and child": "Cuerpo entre masculino, femenino y niño",
  "drag between male, female and child; arrow keys step":
    "arrastra entre masculino, femenino y niño; las flechas avanzan a pasos",

  // ---------------------------------------------------------------- speak
  Say: "Decir",
  "Text to speech": "Texto a voz",
  "What to say": "Qué decir",
  "Text is IPA": "El texto es AFI",
  English: "Inglés",
  Spanish: "Español",
  Dialect: "Dialecto",
  "IPA to speak": "AFI a decir",
  "Text to speak": "Texto a decir",
  "Type IPA, for example ˈhɛloʊ": "Escribe AFI, por ejemplo ˈhɛloʊ",
  "Type something for the voice to say": "Escribe algo para que lo diga la voz",
  "Preparing…": "Preparando…",
  Stop: "Detener",
  "The transcription appears here.": "La transcripción aparece aquí.",
  "Not said": "No se dice",
  "No sound yet": "Aún sin sonido",
  "Mark ignored": "Marca ignorada",
  Rate: "Velocidad",
  Examples: "Ejemplos",
  "Gestural score": "Partitura gestual",
  Consonants: "Consonantes",
  "Between two vowels": "Entre dos vocales",
  "Vowel around it": "Vocal a su alrededor",
  "Say it between two vowels": "Dilo entre dos vocales",

  // the consonant chart's places and manners
  Bilabial: "Bilabial",
  Labiodental: "Labiodental",
  Dental: "Dental",
  Alveolar: "Alveolar",
  Postalveolar: "Postalveolar",
  Retroflex: "Retrofleja",
  Palatal: "Palatal",
  Velar: "Velar",
  Uvular: "Uvular",
  Pharyngeal: "Faringeal",
  Glottal: "Glotal",
  Bilab: "Bilab",
  Labdent: "Labdent",
  Alveol: "Alveol",
  Postalv: "Postalv",
  Retrofl: "Retrofl",
  Pharyn: "Farín",
  Plosive: "Oclusiva",
  Nasal: "Nasal",
  Trill: "Vibrante",
  Tap: "Batiente",
  Fricative: "Fricativa",
  "Lateral fricative": "Fricativa lateral",
  Approximant: "Aproximante",
  Lateral: "Lateral",
  "Lat fric": "Fric lat",
  Approx: "Aprox",

  // the face's readouts (speak)
  Sound: "Sonido",
  Jaw: "Mandíbula",
  Lips: "Labios",
  "Soft palate": "Paladar blando",
  Closed: "Cerrado",
  Round: "Redondo",
  Open: "Abierto",
  Spread: "Estirado",
  Shut: "Cerrado",
  Sealed: "Sellado",
  Voiced: "Sonoro",
  Voiceless: "Sordo",
  "Noise at": "Ruido a",
  Lungs: "Pulmones",
  Tract: "Tracto",
  Mouth: "Boca",
  Tongue: "Lengua",

  // ---------------------------------------------------------------- sing
  Note: "Nota",
  Syllable: "Sílaba",
  "Ring on": "Ring activo",
  "Ring off": "Ring inactivo",
  "Lyrics to sing": "Letra para cantar",
  "Type the words to sing": "Escribe las palabras para cantar",
  "Each note sings the next syllable. Hold a key while you press the next one to sing legato. A lone “-” between words holds a vowel over several notes; hyphens split a word your way (“ti-ger”).":
    "Cada nota canta la siguiente sílaba. Mantén una tecla mientras pulsas la siguiente para cantar legato. Un «-» solo entre palabras sostiene una vocal en varias notas; los guiones parten la palabra a tu manera («ti-ger»).",
  "Syllables. Each note sings the marked one; choose another to start there.":
    "Sílabas. Cada nota canta la marcada; elige otra para empezar ahí.",
  "Held vowel: carries the previous syllable onto this note":
    "Vocal sostenida: lleva la sílaba anterior a esta nota",
  "Sing from": "Cantar desde",
  "No words: the keys sing “ah”.": "Sin palabras: las teclas cantan «ah».",
  "From the start": "Desde el principio",
  "Song voice": "Voz de canto",
  Song: "Canción",
  Tone: "Timbre",
  Vibrato: "Vibrato",
  Breathy: "Aliento",
  Bright: "Brillo",
  "Piano roll": "Piano roll",
  "Notes and the pitch sung": "Las notas y la altura cantada",
  Lyrics: "Letra",
  "Each note sings the next syllable": "Cada nota canta la siguiente sílaba",
  Next: "Siguiente",
  "Demo songs": "Canciones de ejemplo",

  // ---------------------------------------------------------------- beatbox
  "Breath out": "Aire afuera",
  "Larynx piston": "Pistón laríngeo",
  "Tongue suction": "Succión de lengua",
  "Breath in": "Aire adentro",
  Larynx: "Laringe",
  "Beat pattern: one sound per step": "Patrón de ritmo: un sonido por paso",
  step: "paso",
  Step: "Paso",
  "Play beat": "Tocar ritmo",
  Beat: "Ritmo",
  Pattern: "Patrón",
  Bass: "Bajo",
  "Mic FX": "FX de micro",
  "Close-mic effect: the bass, presence and squash of a mic held to the lips":
    "Efecto de micro cercano: el bajo, la presencia y la compresión de un micro pegado a los labios",
  "Close mic": "Cerrar micrófono",
  Transport: "Transporte",
  Tempo: "Tempo",
  Swing: "Swing",
  "Example beats": "Ritmos de ejemplo",
  "Beat as text": "Ritmo como texto",
  "B kick, t hat, Pf snare, K rimshot, ^K inward, ! click, m hum (m:43 sets its note), . rest":
    "B bombo, t charles, Pf caja, K aro, ^K caja inversa, ¡ chasquido, m tarareo (m:43 fija su nota), . silencio",
  "Step sequencer": "Secuenciador de pasos",
  "Your beat": "Tu ritmo",
  Spectrogram: "Espectrograma",
  "Last two seconds / newest in front": "Los últimos dos segundos / lo más nuevo al frente",
  Pads: "Almohadillas",
  "Keys 1 to 0": "Teclas 1 a 0",
  "Hum a bass note": "Tararea una nota de bajo",

  // beat sounds (names come from the engine's catalogue; translated here at the display)
  Kick: "Bombo",
  "Lips burst on air the rising larynx compresses":
    "Estallido de los labios sobre aire que comprime la laringe al subir",
  "Hi-hat": "Charles",
  "Tongue-tip release into a short hiss": "Suelta de la punta de la lengua en un siseo corto",
  "Open hat": "Charles abierto",
  "The hi-hat left ringing as a long hiss": "El charles dejado sonando como un siseo largo",
  Snare: "Caja",
  "Lips burst, then the lower lip hisses against the teeth":
    "Estallido de los labios, luego el labio inferior sisea contra los dientes",
  Rimshot: "Aro",
  "Back of the tongue bursts on larynx-compressed air":
    "El fondo de la lengua estalla sobre aire comprimido por la laringe",
  "Inward snare": "Caja inversa",
  "A sharp breath in past the back of the tongue":
    "Una entrada de aire brusca junto al fondo de la lengua",
  "Tongue click": "Chasquido de lengua",
  "Tongue seals a pocket, hollows it and pops": "La lengua sella una bolsa, la ahueca y la revienta",
  "Lip pop": "Pop de labios",
  "Sealed lips, the jaw drops, the lips pop open":
    "Labios sellados, la mandíbula baja, los labios se abren de golpe",
  Crash: "Platillo",
  "Back-of-tongue release into a long rounded hiss":
    "Suelta del fondo de la lengua en un siseo largo y redondeado",
  Hum: "Tarareo",
  "Hummed bass note through the nose": "Nota de bajo tarareada por la nariz",

  // beat presets
  "Boots and cats": "Botas y gatos",
  "Four on the floor": "Cuatro en el suelo",
  "Rimshot shuffle": "Shuffle de aro",
  "Clicks and pops": "Chasquidos y pops",
  "Hum bass": "Bajo tarareado",

  // ---------------------------------------------------------------- bestiary
  "Creatures": "Criaturas",
  specimens: "ejemplares",
  "Waterfall of the call": "Cascada del llamado",
  Calls: "Llamados",
  Controls: "Controles",
  controls: "controles",
  Airway: "Vía aérea",
  "Each tube unrolled": "Cada tubo desenrollado",
  "The held voice or a call, at the key's pitch": "La voz sostenida o un llamado, a la altura de la tecla",
  "Keep the breath on and shape the sound with the controls":
    "Mantén el aire encendido y moldea el sonido con los controles",
  "Stop sustaining": "Dejar de sostener",
  Sustain: "Sostener",
  "What the keys play": "Lo que tocan las teclas",
  "Keys play": "Las teclas tocan",
  "Held voice": "Voz sostenida",
  Sustaining: "Sosteniendo",
  "At rest": "En reposo",
  "Not started": "Sin iniciar",

  // creatures (names and display strings from the engine's catalogue)
  Songbird: "Ave cantora",
  "Two independent labial pairs in the syrinx, one per bronchus, each with its own pitch; a throat cavity tracks the note.":
    "Dos pares labiales independientes en la siringe, uno por bronquio, cada uno con su propio tono; una cavidad de garganta sigue la nota.",
  "Shimmering two-note leads, arpeggio trills, glitchy chirps":
    "Leads de dos notas con brillo, trinos en arpegio, gorjeos glitch",
  "Weddell seal": "Foca de Weddell",
  "Laryngeal calls with an extreme range: trills glide down for seconds while their pulses slow.":
    "Llamados laríngeos de rango extremo: los trinos descienden durante segundos mientras sus pulsos se lentifican.",
  "Downlifters, laser zaps, acid-style sweeping leads":
    "Descensos, zaps láser, leads barriendo estilo acid",
  "American alligator": "Caimán de Mississippi",
  "A chaotic, very low laryngeal roar through a long, closed-mouthed tract; the whole body radiates.":
    "Un rugido laríngeo caótico y muy grave por un tracto largo y con la boca cerrada; todo el cuerpo irradia.",
  "Distorted sub drones, noise bass, impacts": "Drones sub distorsionados, bajos de ruido, impactos",
  "African elephant": "Elefante africano",
  "Infrasonic rumbles from huge vocal folds, routed through a two-metre trunk or the mouth; trumpets from a buzzing valve at the trunk entrance.":
    "Retumbos infrasonoros de cuerdas vocales enormes, enrutados por una trompa de dos metros o por la boca; trompeteos desde una válvula que zumba a la entrada de la trompa.",
  "Sub-bass rumbles, distorted brass leads": "Retumbos de subgraves, leads de metal distorsionado",

  // creature calls
  Bellow: "Bramido",
  "Two seconds of chaotic roar, rising into full chaos":
    "Dos segundos de rugido caótico que sube hacia el caos pleno",
  Growl: "Gruñido",
  "Short, rough warning": "Advertencia corta y áspera",
  Hiss: "Siseo",
  "Open mouth, open glottis: pure turbulence": "Boca abierta, glotis abierta: turbulencia pura",
  "Water dance": "Danza del agua",
  "Steady subsonic throb that shakes the water": "Palpitación subsónica constante que agita el agua",
  Rumble: "Retumbo",
  "Three seconds of infrasonic rumble through the trunk":
    "Tres segundos de retumbo infrasonoro por la trompa",
  Trumpet: "Trompeteo",
  "A brassy blast that bends up and breaks into chaos":
    "Una detonación metálica que sube y se rompe en caos",
  Squeak: "Chirrido",
  "Tensed, high valve buzz": "Zumbido agudo y tenso de la válvula",
  "Descending trill": "Trino descendente",
  "Several seconds falling from bright to subsonic, pulses slowing":
    "Varios segundos cayendo de lo brillante a lo subsónico, con pulsos que se lentifican",
  "Laser chirp": "Gorjeo láser",
  "A 150 ms downward zap": "Un zap descendente de 150 ms",
  "U whistle": "Silbido en U",
  "Dips and returns": "Baja y regresa",
  Chug: "Traqueteo",
  "Low, pulsed grunt": "Grunido grave y pulsado",
  "Flute phrase": "Frase de flauta",
  "Wood-thrush style: three notes, then a two-voice shimmer":
    "Estilo zorzal del bosque: tres notas, y luego un brillo a dos voces",
  "Canary trill": "Trino de canario",
  "16 notes a second, each a quick upsweep": "16 notas por segundo, cada una con una subida rápida",
  "Chirp cascade": "Cascada de gorjeos",
  "Falling chirps, faster and faster": "Gorjeos descendentes, cada vez más rápidos",
  "Finch buzz": "Zumbido de pinzón",
  "Zebra-finch style: raspy, pulsed by mini-breaths":
    "Estilo pinzón cebra: áspero, pulsado por mini-respiraciones",

  // creature parameters
  "Bifurcation (period doubling → chaos)": "Bifurcación (duplicación de período → caos)",
  "Breath pressure": "Presión de aire",
  "Mouth opening": "Apertura de la boca",
  "Nasal passage": "Pasaje nasal",
  "Hiss (open glottis)": "Siseo (glotis abierta)",
  "Rumble ↔ trumpet": "Retumbo ↔ trompeteo",
  "Rumble pitch": "Tono del retumbo",
  "Trumpet valve pitch": "Tono de la válvula de trompeteo",
  "Mouth ↔ trunk": "Boca ↔ trompa",
  Chaos: "Caos",
  "Pulse rate": "Tasa de pulsos",
  "Pulsing depth": "Profundidad del pulso",
  "Pulse duty": "Ciclo del pulso",
  Brightness: "Brillo",
  Roughness: "Rugosidad",
  "Pitch, left voice": "Tono, voz izquierda",
  "Right voice interval": "Intervalo de la voz derecha",
  "Left ↔ both ↔ right": "Izquierda ↔ ambas ↔ derecha",
  "Drive (pure → raspy)": "Distorsión (puro → áspero)",
  "Labial tension": "Tensión labial",
  "Air-sac pressure": "Presión del saco aéreo",
  "Throat tracks note": "La garganta sigue la nota",
  "Throat cavity size": "Tamaño de la cavidad de la garganta",
  "Beak gape": "Apertura del pico",

  // creature airway and source labels (creatureVisuals.ts)
  "Left bronchus": "Bronquio izquierdo",
  "Right bronchus": "Bronquio derecho",
  Trachea: "Tráquea",
  Beak: "Pico",
  "Left voice": "Voz izquierda",
  "Right voice": "Voz derecha",
  Pharynx: "Faringe",
  Nostrils: "Fosas nasales",
  Trunk: "Trompa",
  "Trunk valve": "Válvula de la trompa",

  // ---------------------------------------------------------------- face and instruments (canvas)
  CLOSED: "CERRADO",
  "Tongue body": "Cuerpo de la lengua",
  "Tongue tip": "Punta de la lengua",
  Glottis: "Glotis",
  FRONT: "FRENTE",
  AREA: "ÁREA",
  FLOW: "FLUJO",
  "OPEN": "ABIERTO",
  "MAKE A SOUND TO SEE THEM VIBRATE": "HAZ UN SONIDO PARA VERLAS VIBRAR",
  "START AUDIO TO SEE THEM VIBRATE": "INICIA EL AUDIO PARA VERLAS VIBRAR",
  NOSE: "NARIZ",
  "VELUM OPEN": "VELO ABIERTO",
  "VELUM SHUT": "VELO CERRADO",
  GLOTTIS: "GLOTIS",
  VELUM: "VELO",
  TEETH: "DIENTES",
  LIPS: "LABIOS",
  NOW: "AHORA",
  "S AGO": "S ATRÁS",
  "START AUDIO TO SEE THE SOUND": "INICIA EL AUDIO PARA VER EL SONIDO",
  PITCH: "TONO",
  "SPEAK SOMETHING TO SEE ITS GESTURES: WHICH ARTICULATOR MOVES, WHEN, AND HOW THEY OVERLAP.":
    "DI ALGO PARA VER SUS GESTOS: QUÉ ARTICULADOR SE MUEVE, CUÁNDO, Y CÓMO SE SUPERPONEN.",
  LOWERED: "BAJADO",

  // ---------------------------------------------------------------- keys, MIDI, voice controls
  "Keyboard. Computer keys A to semicolon play notes; Z and X change octave.":
    "Teclado. Las teclas de la A al punto y coma tocan notas; Z y X cambian de octava.",
  "Connecting…": "Conectando…",
  "Blocked by the browser": "Bloqueado por el navegador",
  "Plug one in": "Conecta uno",
  Off: "Apagado",
  "MIDI keyboard": "Teclado MIDI",
  "Disconnect the MIDI keyboard": "Desconecta el teclado MIDI",
  "Press to disconnect.": "Pulsa para desconectar.",
  "Press to connect (the browser asks for access).": "Pulsa para conectar (el navegador pide acceso).",
  "Play with a MIDI keyboard (the browser asks for access)":
    "Toca con un teclado MIDI (el navegador pide acceso)",

  Pitch: "Tono",
  Breath: "Aire",
  "Breath mark": "Respiración",
  Pressed: "Tensión",
  Rough: "Rugoso",
  Fry: "Fry",
  Glide: "Deslizamiento",
  Rounding: "Redondeo",
  "Lips open": "Apertura labial",
  Protrude: "Protracción",
  Root: "Raíz",
  "Tip curl": "Curva de la punta",
  Throat: "Garganta",
  Twang: "Twang",
  "False folds": "Cuerdas falsas",
  "Hold breath": "Aguantar el aire",
  Whisper: "Susurro",

  // ---------------------------------------------------------------- duet page
  "Loading…": "Cargando…",
  "Loading the song…": "Cargando la canción…",
  Pause: "Pausa",
  Again: "Otra vez",
  "Play the song": "Reproducir",
  Key: "Tecla",
  "Position in the song": "Posición en la canción",
  "This show could not load": "No se pudo cargar este show",
  voice: "voz",
  "Vocal tract cross-section. Drag the tongue, tongue tip, jaw, lips, soft palate or larynx handles.":
    "Corte del tracto vocal. Arrastra las asas de la lengua, la punta de la lengua, la mandíbula, los labios, el paladar blando o la laringe.",
  "Vocal tract in 3D. Drag to turn the head, scroll or pinch to zoom, double-click to go back to the starting view.":
    "Tracto vocal en 3D. Arrastra para girar la cabeza, rueda o pellizca para acercar, doble clic para volver a la vista inicial.",
  "Vowel map. Drag to morph between vowels.": "Mapa de vocales. Arrastra para pasar de una vocal a otra.",
  "vowel position": "posición de la vocal",
  "Piano roll: the notes played, their syllables, and the pitch actually sung":
    "Piano roll: las notas tocadas, sus sílabas y la altura realmente cantada",
  "Gestural score of the current utterance: which articulator moves, when, and how they overlap":
    "Partitura gestual del enunciado actual: qué articulador se mueve, cuándo, y cómo se superponen",
  "Creature airway: its outline, its airways lit by sound pressure, and its sound sources":
    "Vía aérea de la criatura: su contorno, sus vías aéreas iluminadas por la presión sonora y sus fuentes sonoras",
  "Airway tubes unrolled, lit by sound pressure": "Tubos de la vía aérea desenrollados, iluminados por la presión sonora",
  "Waterfall spectrogram of the last two seconds of output":
    "Espectrograma en cascada de los últimos dos segundos de salida",
  "Output spectrum: harmonics, the vocal tract's resonances and its formants":
    "Espectro de salida: armónicos, las resonancias del tracto vocal y sus formantes",
  "Area function of the simulated tube, lit by sound pressure":
    "Función de área del tubo simulado, iluminada por la presión sonora",
  "Vocal folds seen from above, replayed slowly, with the glottal area and flow":
    "Cuerdas vocales vistas desde arriba, repetidas a cámara lenta, con el área y el flujo glotales",
  Tune: "Melodía",
  "Tonal key": "Tonalidad",
  semitones: "semitonos",
  Ring: "Anillo vocal",
  "Recording…": "Grabando…",
  "Record video": "Grabar vídeo",
  "Sing it again and record the face into a video": "Canta de nuevo y graba la cabeza en vídeo",
  Cancel: "Cancelar",
  Share: "Compartir",
  Save: "Guardar",
  "Sing sings these words to the tune, one line per phrase. Each key sings the next syllable; hold keys for legato. A lone dash holds a vowel; hyphens split a word.": "Canta estas palabras con la melod?a, una l?nea por frase. Cada tecla canta la s?laba siguiente; mant?n una tecla mientras pulsas otra para ligar las notas. Un guion aislado prolonga una vocal; los guiones dentro de una palabra separan s?labas.",
};
