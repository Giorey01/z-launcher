# Archivi RAR di prova

Casi letti da `fixtures/archive/rar.json` (contenuto atteso: sha256 di ogni file estratto con l'UnRAR
ufficiale, indipendente dal lettore Rust `rars` sotto test).

| Cartella | Come è fatto |
|---|---|
| `rar5-single` | RAR 5, un file solo (`rar a -ma5 -m3`) |
| `rar5-volumes` | RAR 5 in 4 volumi da 64 KB (`-v64k`), schema `set.partN.rar` |
| `rar5-solid-volumes` | come sopra ma solid (`-s`) |
| `rar5-password` | 4 volumi con password `Secret123` (`-p`) |
| `rar5-headers` | intestazioni cifrate, anche i nomi dei file (`-hp`), password `Secret123` |
| `rar4-volumes` | RAR 4 in 7 volumi, schema vecchio `set.rar`, `set.r00` … `set.r05` |
| `rar4-solid` | RAR 4 solid, un file |

I set `rar5-*` li rigenera `make.sh` con RAR 7.23 (www.rarlab.com; il programma non sta nel
repository): `bash fixtures/archive/rar/make.sh /percorso/della/cartella/rar`. Il contenuto viene da
`content.py` (deterministico), il JSON da `expected.py`.

I set `rar4-*` vengono da HatchZip (https://github.com/xevrion/hatchzip,
`crates/cli/tests/fixtures/archive/rar/rar4` e `rar4-solid`, MIT OR Apache-2.0, fatti con RAR 6.24):
RAR 7 non scrive più il formato RAR 4.
