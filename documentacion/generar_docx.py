"""Convierte documentacion/guion_video.md en un archivo Word (.docx).

Uso:
    python documentacion/generar_docx.py

Genera: documentacion/guion_video.docx
"""

from pathlib import Path
import re

from docx import Document
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
from docx.shared import Cm, Pt, RGBColor

BASE = Path(__file__).resolve().parent
ORIGEN = BASE / "guion_video.md"
DESTINO = BASE / "guion_video.docx"

TOKENES = re.compile(r"(\*\*.+?\*\*|`[^`]+`)")


def añadir_texto_enriquecido(parrafo, texto):
    """Acepta **negrita**, `código` y *cursiva* dentro de una línea Markdown."""
    for parte in TOKENES.split(texto):
        if not parte:
            continue
        if parte.startswith("**") and parte.endswith("**"):
            run = parrafo.add_run(parte[2:-2])
            run.bold = True
        elif parte.startswith("`") and parte.endswith("`"):
            run = parrafo.add_run(parte[1:-1])
            run.font.name = "Consolas"
            run.font.size = Pt(9.5)
            run.font.color.rgb = RGBColor(0xB0, 0x30, 0x60)
        else:
            # Cursiva sencilla: *texto*
            for trozo in re.split(r"(\*.+?\*)", parte):
                if not trozo:
                    continue
                if trozo.startswith("*") and trozo.endswith("*") and len(trozo) > 2:
                    run = parrafo.add_run(trozo[1:-1])
                    run.italic = True
                else:
                    parrafo.add_run(trozo)


def sombrear(parrafo, color="F2F2F2"):
    props = parrafo._p.get_or_add_pPr()
    shd = OxmlElement("w:shd")
    shd.set(qn("w:fill"), color)
    props.append(shd)


def configurar_estilos(doc):
    normal = doc.styles["Normal"]
    normal.font.name = "Calibri"
    normal.font.size = Pt(11)
    normal.paragraph_format.space_after = Pt(6)

    for nombre, tamano, color in (
        ("Heading 1", 18, RGBColor(0x1F, 0x4E, 0x79)),
        ("Heading 2", 14, RGBColor(0x2E, 0x74, 0xB5)),
        ("Heading 3", 12, RGBColor(0x40, 0x40, 0x40)),
    ):
        estilo = doc.styles[nombre]
        estilo.font.name = "Calibri"
        estilo.font.size = Pt(tamano)
        estilo.font.bold = True
        estilo.font.color.rgb = color
        estilo.paragraph_format.space_before = Pt(12)
        estilo.paragraph_format.space_after = Pt(6)


def es_separador(linea):
    celdas = linea.strip().strip("|").split("|")
    return bool(celdas) and all(set(c.strip()) <= set("-: ") for c in celdas)


def agregar_tabla(doc, filas):
    celdas = [
        [celda.strip() for celda in fila.strip().strip("|").split("|")]
        for fila in filas
        if not es_separador(fila)
    ]
    if not celdas:
        return
    tabla = doc.add_table(rows=len(celdas), cols=len(celdas[0]))
    tabla.style = "Table Grid"
    tabla.alignment = WD_ALIGN_PARAGRAPH.LEFT
    for i, fila in enumerate(celdas):
        for j, valor in enumerate(fila):
            celda = tabla.cell(i, j)
            celda.text = ""
            parrafo = celda.paragraphs[0]
            añadir_texto_enriquecido(parrafo, valor)
            if i == 0:
                for run in parrafo.runs:
                    run.bold = True
    doc.add_paragraph()


def main():
    lineas = ORIGEN.read_text(encoding="utf-8").splitlines()

    doc = Document()
    seccion = doc.sections[0]
    seccion.top_margin = Cm(2)
    seccion.bottom_margin = Cm(2)
    seccion.left_margin = Cm(2.2)
    seccion.right_margin = Cm(2.2)
    configurar_estilos(doc)

    # Pie de página con los autores.
    pie = seccion.footer.paragraphs[0]
    pie.text = "servidor-https-rust  ·  Francisco Sevilla y Yeimar Gonzalez"
    pie.alignment = WD_ALIGN_PARAGRAPH.CENTER
    for run in pie.runs:
        run.font.size = Pt(8)
        run.font.color.rgb = RGBColor(0x77, 0x77, 0x77)

    i = 0
    while i < len(lineas):
        linea = lineas[i]
        texto = linea.strip()

        if not texto or texto == "---":
            i += 1
            continue

        if texto.startswith("```"):
            i += 1
            bloque = []
            while i < len(lineas) and not lineas[i].strip().startswith("```"):
                bloque.append(lineas[i])
                i += 1
            i += 1  # cierre del bloque
            for codigo in bloque:
                p = doc.add_paragraph()
                p.paragraph_format.left_indent = Cm(0.6)
                p.paragraph_format.space_after = Pt(0)
                run = p.add_run(codigo if codigo else " ")
                run.font.name = "Consolas"
                run.font.size = Pt(9)
                sombrear(p)
            doc.add_paragraph()
            continue

        if texto.startswith("|"):
            filas = []
            while i < len(lineas) and lineas[i].strip().startswith("|"):
                filas.append(lineas[i])
                i += 1
            agregar_tabla(doc, filas)
            continue

        if texto.startswith("### "):
            doc.add_heading(texto[4:].replace("`", ""), level=3)
        elif texto.startswith("## "):
            doc.add_heading(texto[3:].replace("`", ""), level=2)
        elif texto.startswith("# "):
            doc.add_heading(texto[2:].replace("`", ""), level=1)

        elif texto.startswith("**DI ("):
            # Unimos la etiqueta "DI (quien)" con el texto citado que viene debajo.
            etiqueta = texto
            j = i + 1
            while j < len(lineas) and not lineas[j].strip():
                j += 1
            cita = ""
            if j < len(lineas) and lineas[j].strip().startswith("> "):
                cita = lineas[j].strip()[2:]
                i = j + 1
            else:
                i += 1
            p = doc.add_paragraph()
            p.paragraph_format.left_indent = Cm(0.4)
            añadir_texto_enriquecido(p, etiqueta + " ")
            if cita:
                añadir_texto_enriquecido(p, cita)

        elif texto.startswith("> "):
            p = doc.add_paragraph()
            p.paragraph_format.left_indent = Cm(0.6)
            añadir_texto_enriquecido(p, texto[2:])
            for run in p.runs:
                run.italic = True
                run.font.color.rgb = RGBColor(0x55, 0x55, 0x55)

        elif texto.startswith("- [x] ") or texto.startswith("- [ ] "):
            marcado = texto.startswith("- [x]")
            p = doc.add_paragraph(style="List Bullet")
            run = p.add_run("☑ " if marcado else "☐ ")
            run.bold = True
            añadir_texto_enriquecido(p, texto[6:])

        elif texto.startswith("- "):
            p = doc.add_paragraph(style="List Bullet")
            añadir_texto_enriquecido(p, texto[2:])

        else:
            p = doc.add_paragraph()
            añadir_texto_enriquecido(p, texto)

        i += 1

    props = doc.core_properties
    props.title = "Guion del video: servidor HTTPS multihilo en Rust"
    props.author = "Francisco Sevilla y Yeimar Gonzalez"
    props.subject = "Explicación del código de servidor-https-rust"
    props.keywords = "Rust, HTTPS, rustls, hilos, guion, video"

    doc.save(DESTINO)
    print(f"Generado: {DESTINO}")


if __name__ == "__main__":
    main()
