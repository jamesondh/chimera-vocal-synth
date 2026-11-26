# Matriz de escucha española

Esta matriz convierte los comentarios de revisión en casos sintéticos reproducibles. No usa audio ni texto protegido ajeno.

## Preparación

```bash
chimera syllables "FRASES" es --dialect neutro
chimera syllables "FRASES" es --dialect rioplatense
chimera syllables "FRASES" es --dialect castellano
chimera render spanish out/español-NEUTRO "FRASES" male --dialect neutro
chimera render spanish out/español-RIOPLATENSE "FRASES" male --dialect rioplatense
chimera render spanish out/español-CASTELLANO "FRASES" male --dialect castellano
```

Cada variante debe escucharse con una frase única por muestra y sin cambiar velocidad, tono, cuerpo o voz.

## Casos priorizados

| Prioridad | Frases | Dialecto | Lo que se busca | Resultado nativo |
|---|---|---|---|---|
| 1 | chico, chica, chino, chocolate | neutro/castellano | claridad de /tʃ/ y ausencia de /ʃ/ |  |
| 1 | ratón, tierra, chocolate | neutro/castellano | ausencia de clics en /t/, /r/ y /rr/ |  |
| 1 | La guerra en la tierra del país | neutro/castellano | acumulación de ruidos, trinos y cierre de palabras |  |
| 2 | caldo, palta, alto | neutro/castellano | liberación lateral normal o ruido superior |  |
| 2 | crecí | neutro/castellano/rioplatense | sílaba inicial completa y sin debilidad excesiva |  |
| 2 | Córdoba | castellano | /ð/ dental y /β/ bilabial, no "Córdofa" |  |
| 2 | higuera | neutro/castellano | /ɣ/ velar, no /x/ ni respiración extra antes de /i/ |  |
| 2 | definía | neutro/castellano/rioplatense | inteligibilidad del hiato y acento |  |
| 3 | llueve, llave, lluvia, mayo | rioplatense/neutro/castellano | yeísmo, sheísmo y ausencia de estridencia |  |

## Registro por caso

Para cada muestra, registrar sólo hechos verificables:

- **Esperado:** la palabra que debe escucharse.
- **Percibido:** la palabra o sonido que realmente se escucha.
- **Lugar:** inicio, sílaba, consonante o transición.
- **Artefacto:** clic, respiración, choque, fricción, silencio, debilidad o cambio de fonema.
- **Variante mejor:** neutro, rioplatense, castellano o ninguna.
- **Comparación:** nivel entre 1 (ininteligible) y 5 (natural y claro).

Ejemplo:

```text
Esperado: higuera
Percibido: higuera / hijera / higuera con respiración
Lugar: antes de i y en la consonante gu
Artefacto: aspiración extra
Variante mejor: ninguna
Nivel: 3
```
