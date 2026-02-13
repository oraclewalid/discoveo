# Discoveo MVP Roadmap

## Overview

**Goal:** Automated CRO insights - funnel drop-offs + AI explanation of "why"

**Timeline:** 4 weeks

**Tech Stack:** Rust + Axum + PostgreSQL (backend), Vue (frontend), Auth0, self-hosted

---

## Week 1: GA4 OAuth + Data Ingestion

### Backend (Rust/Axum)

- [ ] Google Cloud project setup
  - Create project
  - Enable GA4 Data API
  - Configure OAuth consent screen
  - Create OAuth credentials (client ID + secret)
- [ ] OAuth endpoints
  - `GET /auth/ga4/connect` - redirect to Google
  - `GET /auth/ga4/callback` - handle code exchange
  - Store refresh token in PostgreSQL
- [ ] GA4 Data API service
  - List available properties (accounts)
  - Pull events (date range, pagination)
  - Token refresh logic
- [ ] Event storage
  - Events table (event_name, timestamp, user_id, properties JSONB)
  - Sessions table
  - User properties table
- [ ] Background job: sync events on schedule

### Frontend (Vue)

- [ ] App scaffold (Vue Router, basic layout)
- [ ] Connect GA4 page
  - "Connect" button → OAuth redirect
  - Property selector dropdown
  - Connection status indicator
- [ ] Simple dashboard shell (sidebar + main area)

---

## Week 2: Funnel Visualization + Qualitative Data

### Backend

- [ ] Funnel SQL queries (hardcoded/predefined)
  - Ecommerce funnel: view → add_to_cart → checkout → purchase
  - Step-by-step conversion calculation
  - Drop-off % per step
  - User count per step
- [ ] Funnel endpoint
  - `GET /funnels/:id` - returns calculated funnel with drop-offs
- [ ] Qualitative data endpoints
  - `POST /qualitative/upload` - CSV upload
  - `GET /qualitative` - list uploaded data
  - `DELETE /qualitative/:id`
- [ ] Qualitative schema
  - source (survey, interview, usability_test)
  - date
  - content (text)
  - tags (JSONB)

### Frontend

- [ ] Funnel visualization page (read-only)
  - Visual funnel (bars or steps showing %)
  - Drop-off highlighted in red
  - User counts per step
- [ ] Qualitative upload page
  - File dropzone
  - List of uploaded files  

---

## Week 3: AI Analysis + Severity

### Backend

- [ ] AI analysis service
  - Input: funnel data + qualitative data
  - Prompt: explain drop-offs, find patterns
  - Output: structured insights (JSON)
- [ ] Keyword matching for qualitative data
  - Search transcripts for terms related to each funnel step
  - Find relevant quotes for each drop-off
- [ ] Severity scoring
  - Critical: >30% drop-off AND >500 users affected
  - High: >20% drop-off OR strong qual signal
  - Medium: >10% drop-off
  - Low: <10%, optimization opportunity
- [ ] Recommendations endpoints
  - `GET /recommendations` - list all, filter by severity
  - `GET /recommendations/:id` - detail
  - `PATCH /recommendations/:id` - mark resolved/dismissed
- [ ] Recommendations table
  - funnel_id, step, severity, insight_text, qual_evidence, suggested_action, status

### Frontend

- [ ] Recommendations dashboard
  - Filter tabs: All / Critical / High / Medium / Low
  - Cards showing: title, severity badge, affected users
- [ ] Recommendation detail page
  - Funnel step context (mini visualization)
  - AI explanation
  - Qualitative evidence quotes
  - Suggested action
  - Resolve / Dismiss buttons
- [ ] Integrate into funnel detail
  - Show AI insight inline per step with drop-off

---

## Week 4: Auth0 + Aha Moment Foundation + Polish

### Backend

- [ ] Auth0 integration
  - Create Auth0 tenant + application
  - JWT validation middleware
  - Protected routes
- [ ] Multi-tenant prep
  - Workspace table
  - Link all data to workspace_id
- [ ] Aha moment analysis (abstract)
  - Endpoint: `POST /analysis/aha` - run analysis
  - Endpoint: `GET /analysis/aha/results` - get findings
  - Logic: TBD based on use case (ecommerce vs product)

### Frontend

- [ ] Auth0 SDK integration
  - Login / Logout flow
  - Protected routes
  - User profile display
- [ ] Aha moment page (placeholder)
  - Run analysis button
  - Results display (format TBD)
- [ ] Polish
  - Loading states
  - Error handling
  - Empty states
  - Design partner feedback fixes

---

## Summary

| Week | Focus | Deliverable |
|------|-------|-------------|
| 1 | GA4 OAuth + data | Partners can connect their GA4 |
| 2 | Funnel visualization + CSV upload | See drop-offs + upload qualitative data |
| 3 | AI analysis + severity | Prioritized recommendations with "why" |
| 4 | Auth0 + Aha (TBD) + polish | MVP launch-ready |
