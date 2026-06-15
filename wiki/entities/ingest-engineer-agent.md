---
type: entity
title: Ingest Engineer Agent
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Ingest Engineer Agent

Type: AGENT

## From [[obolus-micro04|obolus-micro04]] (2026-06-09)
- responsible for the entire import process of Vectron documentation into the knowledge base
- ensures PDFs, TXT, and HTML files are cleanly converted to text
- writes documents into documents + document_chunks of the PostgreSQL database
- calculates and stores embeddings for chunks
- detects and logs error cases (corrupt PDFs, encoding problems, missing OCR)
- classifies documents
- extracts text from various file types
- chunks text into meaningful sections
- performs DB write operations
- handles logging and error management

## From [[obolus-micro05|obolus-micro05]] (2026-06-09)
- Works closely with RAG DB Agent.
- Works closely with Bot Integrator Agent.
