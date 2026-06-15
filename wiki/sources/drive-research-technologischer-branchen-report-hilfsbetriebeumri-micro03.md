---
type: source
title: drive-research-technologischer-branchen-report-hilfsbetriebeumri-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-technologischer-branchen-report-hilfsbetriebeumri-micro03

Ingested source summary (2026-06-09).

## Entities
- [[en-50129|EN 50129]] (CONCEPT)
- [[fmea|FMEA]] (CONCEPT)
- [[lifecycle-costs|Lifecycle Costs]] (CONCEPT)
- [[hvac-systemen|HVAC-Systemen]] (CONCEPT)
- [[cots-software|COTS-Software]] (CONCEPT)
- [[en-61373|EN 61373]] (CONCEPT)
- [[ssas|SSAS]] (CONCEPT)
- [[en-50155|EN 50155]] (CONCEPT)
- [[en-50121-3-2|EN 50121-3-2]] (CONCEPT)
- [[afir|AFIR]] (CONCEPT)
- [[sil|SIL]] (CONCEPT)
- [[hbu|HBU]] (CONCEPT)
- [[tsi|TSI]] (CONCEPT)
- [[dspace-scalexio|dSPACE SCALEXIO]] (ORGANIZATION)
- [[xing|Xing]] (TOOL)
- [[mil-hdbk-217f|MIL-HDBK-217F]] (BOOK)
- [[sic-umrichter|SiC-Umrichter]] (CONCEPT)
- [[en-45545-2|EN 45545-2]] (CONCEPT)
- [[kununu|Kununu]] (ORGANIZATION)
- [[plexim|Plexim]] (ORGANIZATION)
- [[rams-software|RAMS-Software]] (TOOL)
- [[iec-tr-62380|IEC TR 62380]] (BOOK)
- [[reliability-workbench|Reliability Workbench]] (TOOL)
- [[iec-62425|IEC 62425]] (CONCEPT)
- [[availability-workbench|Availability Workbench]] (TOOL)
- [[eplan-electric-p8|EPLAN Electric P8]] (TOOL)
- [[vector|Vector]] (ORGANIZATION)
- [[hilfsbetriebeumrichter|Hilfsbetriebeumrichter]] (CONCEPT)
- [[power-hil-phil|Power-HIL (PHIL)]] (TOOL)
- [[bemu-fahrzeuge|BEMU-Fahrzeuge]] (CONCEPT)
- [[altium-designer|Altium Designer]] (TOOL)
- [[iec-62279|IEC 62279]] (CONCEPT)
- [[matlab-simulink|MATLAB/Simulink]] (TOOL)
- [[cenelec-normenwerk|CENELEC-Normenwerk]] (ORGANIZATION)
- [[siliziumkarbid-sic-halbleitern|Siliziumkarbid (SiC)-Halbleitern]] (CONCEPT)
- [[en-50657|EN 50657]] (CONCEPT)
- [[linkedin|LinkedIn]] (TOOL)
- [[plecs|PLECS]] (TOOL)
- [[en-50126|EN 50126]] (CONCEPT)
- [[cae-software|CAE-Software]] (TOOL)
- [[isograph-rams-suite|Isograph RAMS Suite]] (TOOL)
- [[t-v-s-d-fscp|TÜV SÜD FSCP]] (CONCEPT)
- [[iec-62278|IEC 62278]] (CONCEPT)
- [[fta|FTA]] (CONCEPT)
- [[hoppecke-intilion|Hoppecke/Intilion]] (ORGANIZATION)
- [[hara|HARA]] (CONCEPT)
- [[markov-ketten|Markov-Ketten]] (CONCEPT)
- [[fha|FHA]] (CONCEPT)
- [[rheinmetall-sterreich|Rheinmetall Österreich]] (ORGANIZATION)
- [[mttr|MTTR]] (CONCEPT)
- [[en-50159|EN 50159]] (CONCEPT)
- [[systemsimulation|Systemsimulation]] (CONCEPT)
- [[iec-61508|IEC 61508]] (CONCEPT)
- [[en-50128|EN 50128]] (CONCEPT)
- [[spice-simulatoren|SPICE-Simulatoren]] (TOOL)
- [[system-engineer|System Engineer]] (PERSON)
- [[mtbf|MTBF]] (CONCEPT)
- [[ansys-medini-analyze|Ansys Medini Analyze]] (TOOL)
- [[hochvolt-batteriesysteme|Hochvolt-Batteriesysteme]] (CONCEPT)
- [[rams-management|RAMS-Management]] (CONCEPT)

## Relations
- AFIR → RELATED_TO → BEMU-Fahrzeuge
- BEMU-Fahrzeuge → USES → Hoppecke/Intilion
- BEMU-Fahrzeuge → USES → HBU
- BEMU-Fahrzeuge → USES → HVAC-Systemen
- CENELEC-Normenwerk → RELATED_TO → TSI
- RAMS-Management → RELATED_TO → EN 50126
- EN 50126 → RELATED_TO → IEC 62278
- EN 50126 → RELATED_TO → IEC 61508
- EN 50126 → RELATED_TO → SIL
- EN 50129 → USES → FMEA
- EN 50129 → USES → FTA
- EN 50129 → USES → Markov-Ketten
- EN 50128 → RELATED_TO → SSAS
- EN 50128 → RELATED_TO → COTS-Software
- EN 50159 → RELATED_TO → EN 50129
- EN 50155 → RELATED_TO → EN 61373
- EN 50121-3-2 → RELATED_TO → EN 50155
- EN 45545-2 → RELATED_TO → EN 50155
- SiC-Umrichter → USES → CAE-Software
- Altium Designer → PART_OF → CAE-Software
- EPLAN Electric P8 → PART_OF → CAE-Software
- PLECS → AUTHORED_BY → Plexim
- PLECS → RELATED_TO → SPICE-Simulatoren
- PLECS → USES → MATLAB/Simulink
- Ansys Medini Analyze → USES → MIL-HDBK-217F
- Ansys Medini Analyze → USES → IEC TR 62380
- Ansys Medini Analyze → RELATED_TO → MATLAB/Simulink
- Ansys Medini Analyze → USES → HARA
- Ansys Medini Analyze → USES → FMEA
- Ansys Medini Analyze → USES → FTA
- Isograph RAMS Suite → USES → Reliability Workbench
- Isograph RAMS Suite → USES → Availability Workbench
- Power-HIL (PHIL) → USES → dSPACE SCALEXIO
- Power-HIL (PHIL) → USES → Vector
- dSPACE SCALEXIO → PART_OF → Power-HIL (PHIL)
- System Engineer → RELATED_TO → RAMS-Management
- System Engineer → RELATED_TO → TÜV SÜD FSCP
- Rheinmetall Österreich → USES → System Engineer
- LinkedIn → USES → System Engineer
- Xing → USES → System Engineer
- Hilfsbetriebeumrichter → RELATED_TO → Hochvolt-Batteriesysteme
- Siliziumkarbid (SiC)-Halbleitern → RELATED_TO → EN 50155
- CENELEC-Normenwerk → RELATED_TO → EN 50126
