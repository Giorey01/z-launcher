# Cocoon: funzioni annunciate nelle news (riferimento clean-room)

Appunti in parole nostre dalle cinque news pubbliche di `cocoon-shell.com/news` (2.0, "dopo la
2.0", 2.1 beta, 3, 3.1; lette il 26 novembre 2026). Come `cocoon-wiki.md`: servono a fissare il
comportamento da raggiungere, non come testo, immagini o nomi da riusare. La colonna **RF** rimanda
a `docs/ROADMAP.md`; le voci nuove nate da questa lettura sono RF-76 e seguenti.

## Navigazione e cursore

| Comportamento | RF |
| --- | --- |
| Il cursore non supera mai l'elemento quando la lista scorre e lo segue anche durante lo scorrimento a tocco | RF-76 |
| L'inerzia dello scorrimento si ferma appena si usa il pad; in generale si spegne prima | RF-76 |
| Nelle impostazioni l'elemento a fuoco non finisce mai mezzo fuori schermo o sotto le sfumature; lo scorrimento automatico tiene un margine | RF-76 |
| Start chiude le schermate delle impostazioni | RF-36 |
| Suggerimenti dei tasti negli angoli che si nascondono da soli; opzione per tenerli fissi | RF-37 |
| Stile e velocità delle animazioni (logo, sfondo, menu, dialoghi, transizioni) regolabili | RF-77 |
| Menu contestuali e dialoghi che si trasformano l'uno nell'altro invece di aprire finestre nuove | RF-77 |

## Home e griglia

| Comportamento | RF |
| --- | --- |
| Icone più grandi: quella scelta riempie lo spazio; dimensione regolabile | RF-23 |
| Cartelle smart colorate per piattaforma; cartelle con bordo bianco e riempite del loro colore | RF-48 |
| Scorciatoie ai Pod e alle impostazioni direttamente nella griglia, che si tolgono e si rimettono dal menu Start | RF-78 |
| Dock con sfondo, fino a cinque icone, nascosto da solo, in alto o in basso | RF-27 |
| Home a pagine; interruttore per lo scorrimento continuo | RF-24 |
| Modifica della griglia: si entra tenendo premuto A (o il dito); si spostano e ridimensionano le caselle, le vicine fanno spazio; X toglie, con annulla | RF-26, RF-79 |
| Un gioco si ridimensiona come un widget | RF-79 |
| Selezione multipla nella modifica della griglia; "Aggiungi a…" e "Sposta in…" | RF-16 |
| Griglia "Aggiungi giochi": tutti i giochi non ancora nella Home, scelti in blocco | RF-80 |
| Caselle vuote nella griglia; dal loro menu: modifica griglia, nuova cartella, impostazioni | RF-26 |
| Cartelle annidate a ogni profondità, aperte a schermo intero, con posizione e zoom ricordati | RF-15 |
| Cancellare una cartella chiede: sottocartelle, togli i giochi, o spostali al livello sopra | RF-15 |
| Cartelle smart per genere e per espressione sul nome, con suggerimenti per le serie; sottocartelle smart | RF-13 |
| Ordinamento predefinito delle cartelle nuove (A–Z, recenti, più giocati, data di aggiunta) | RF-16 |
| Stili a riga singola: standard, carosello, cover flow | RF-25 |
| Widget nella griglia (orologio, recenti, ultimo giocato, musica, immagine, Log, gioco casuale) e widget Android | RF-64, RF-65 |

## Giochi in corso

| Comportamento | RF |
| --- | --- |
| Home dentro un gioco lo sospende e torna alla Home, dove il gioco resta indicato; A riprende, X chiude; Home di nuovo rientra nel gioco | RF-81 |
| Chiudere un gioco chiude anche le app che gli stanno accanto; più giochi aperti si possono chiudere insieme | RF-81 |
| Sul gioco in corso una X rossa per chiuderlo; i giochi sospesi in basso a sinistra | RF-81 |
| Animazione di lancio: l'artwork cresce fino a riempire gli schermi; alla chiusura si torna sul gioco | RF-82 |
| Sessione che finisce da sola dopo un tempo senza ritorno; sessioni e Now Playing ricostruiti se Android chiude il launcher | RF-40 |
| Il launcher ferma le proprie animazioni e libera memoria mentre si gioca | RF-83 |
| Emulatore chiuso o crashato gestito senza bloccare il launcher | RF-81 |

## Scheda gioco

| Comportamento | RF |
| --- | --- |
| Tre schede: Info (descrizione, video, screenshot, ore HLTB), Achievement, Attività (grafico settimanale) | RF-30, RF-31, RF-32 |
| Stile delle icone: colorate, del colore d'accento, piatte | RF-48 |
| Logo del gioco spostabile e ridimensionabile sopra l'hero, per gioco | RF-84 |
| Scelta dell'artwork da più fonti o da file | RF-22 |
| Hero: schermo intero, vignettatura o niente; scala del logo; movimento del logo | RF-61 |

## Libreria, scraping, app

| Comportamento | RF |
| --- | --- |
| Fonti in ordine di priorità, disattivabili, in parallelo; solo immagini, solo dati o entrambi; mancanti, tutti, una piattaforma | RF-18, RF-19 |
| Anteprima dal vivo nello scraping: nome corretto e ricerca di nuovo, o salto alla scelta dell'artwork | RF-22 |
| Rescan di piattaforme scelte | RF-03 |
| Cartelle che finiscono in `.m3u` (e simili) lanciano il file giusto dentro | RF-02 |
| Scorciatoie esposte da altre app (scorciatoie fissate di Android) usate come giochi, con piattaforma assegnabile | RF-85 |
| Icona dell'app personalizzabile: icona del launcher in un colore o da un icon pack; icona propria nel dock | RF-86 |
| Rilevamento dei permessi delle cartelle persi, con guida per ridarli | RF-87 |

## Sistema

| Comportamento | RF |
| --- | --- |
| Aggiornamento automatico dal launcher, con le novità mostrate prima | RF-88 |
| Profilo di ottimizzazione installato con l'app (avvio veloce dal primo lancio) | RF-83 |
| Opzione per spegnere gli artwork animati, attiva da sola sui dispositivi lenti; GIF ferme se non selezionate | RF-83 |
| Barra di stato con icone che contengono i conteggi; batteria in percentuale, che lampeggia sotto il 10% | RF-68 |
| Pannello di stato (interruttori rapidi e notifiche) e pannello social | RF-68, RF-69 |
| Attesa di qualche secondo dopo il riavvio prima di partire | RF-87 |
| Modalità schermo singolo: nasconde le opzioni solo dual-screen, non chiede lo schermo al lancio | RF-62 |
| Display esterno con passaggio automatico tra layout | RF-62, RF-63 |
| Musica per ora del giorno o in playlist; jingle per gioco al passaggio, anche da repository | RF-52 |

## Pod e impostazioni

| Comportamento | RF |
| --- | --- |
| Impostazioni: categorie a sinistra con ricerca, a destra righe con icona, titolo e sottotitolo; anteprima dal vivo sull'altro schermo | RF-76 |
| Store dei temi: pagina iniziale a grandi card (popolari, nuovi, sfoglia, aggiornamenti); elenco a card con miniatura, autore, download, voti e icone delle parti incluse; schede con LB/RB; scheda dettaglio con anteprime a sinistra e descrizione a destra | RF-89 |
| Log: spostare il tempo di gioco da un gioco a un altro; Select apre la ricerca; grafico settimanale | RF-41, RF-42 |
| Screenshot per gioco con modifica, che restano anche se la ROM sparisce | RF-66 |
| Ogni Pod ha un layout per schermo singolo e per schermi quasi quadrati | RF-89 |
| Tastiera e trackpad sul secondo schermo | RF-39 |
| Collegamento a un server RomM: sfoglia, scarica, cancella | RF-90 |

## Intestazioni comuni delle schermate

Quello che si vede nelle immagini delle news e serve per il nostro stile (descritto, non copiato):

- in alto a sinistra una pillola con il tasto B, una freccia indietro e il titolo;
- in alto a destra pillole con il glifo del tasto e l'icona dell'azione (A scarica, X vota…);
- le schede si cambiano con LB/RB, scritti accanto alla pillola delle schede;
- l'elemento a fuoco è una card con un alone colorato che la contorna;
- in fondo alla lista una freccetta indica che c'è altro sotto.
