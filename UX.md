# Discoveo UX - Navigation Structure

## Current Problems

Current sidebar:
1. **Dashboard** - vague, what data does it show?
2. **AI Recommendation** - sounds like a feature, not a destination
3. **Project Analysis** - actually does project CRUD, misleading name
4. **Surveys** - disconnected from the funnel workflow

The core issue: **the menu is organized around technical features, not the user's workflow**. A CRO person thinks in this order: *see the data -> spot the problem -> understand why -> fix it*.

## Proposed Navigation Structure

```
SIDEBAR
─────────────────────────
[Discoveo logo]

Overview              <- replaces "Dashboard": KPIs, summary of worst drop-offs
Funnels               <- THE core screen: funnel visualization with drop-off %
Qualitative Data      <- replaces "Surveys": upload CSV, interviews, usability tests
Recommendations       <- replaces "AI Recommendation": prioritized by severity

── Settings ──
Projects              <- replaces "Project Analysis": CRUD, GA4 connection
─────────────────────────
[Project switcher at bottom - keep as-is]
```

## Why This Works for CRO Users

| Menu item | Mental model | Maps to roadmap |
|-----------|-------------|-----------------|
| **Overview** | "How is my site doing?" | Week 2: dashboard with key metrics |
| **Funnels** | "Where are users dropping off?" | Week 2: funnel visualization |
| **Qualitative Data** | "What are users saying?" | Week 2: CSV upload, qualitative data |
| **Recommendations** | "What should I fix first?" | Week 3: AI insights + severity |
| **Projects** | Setup/config, not daily use | Week 1: GA4 connect, project management |

## Key Principles

- **Verb-free labels** - nouns are easier to scan ("Funnels" not "Analyze Funnels")
- **Workflow order** - top to bottom follows the analysis flow
- **Settings at the bottom** - project setup is a one-time thing, not a daily action
- **"Funnels" is the hero** - it's the core value prop, so it gets prime position

## Default Landing Page

When a project is already selected, land on **Funnels** directly (not Overview). The funnel screen is where CRO people spend 80% of their time. Overview is a nice summary but secondary.
