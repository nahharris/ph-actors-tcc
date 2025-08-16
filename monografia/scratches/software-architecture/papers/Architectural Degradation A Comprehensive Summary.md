## Overview
This paper presents the first comprehensive multivocal literature review (MLR) on architectural degradation, analyzing 108 studies from 1992-2024 across academic and gray literature to unify understanding of definitions, causes, metrics, tools, and remediation approaches.

## Key Definitions

### Unified Definition of Architectural Degradation
The authors propose a comprehensive definition:

> **Architectural degradation is the progressive divergence between a software system's implemented and intended architecture, caused by repeated violations of architectural decisions, rules, and principles, and by cumulative code-level changes that undermine structural consistency. It entails the loss of key architectural properties, such as modularity, cohesion, and separation of concerns, leading to increased coupling, internal inconsistency, and rising complexity.**

### Evolution of Terminology
- **1992**: Simple "violation of the architecture"
- **2009+**: Expanded to include deviations from design intentions, decisions, and rules
- **2011+**: Characterized as gradual, ongoing decline in architectural quality
- **2019+**: Emphasized loss of structural integrity from design principle violations
- **2022-2024**: Connected to measurable attributes (coupling, cohesion, complexity)

## Root Causes (Motivations)

The study categorizes degradation motivations into three debt types:

### 1. Architectural Debt (56.5% of cases)
- **Architectural Documentation (21.3%)**: Lack of documentation, unclear views, design not in sync
- **Design Issues (12%)**: Architectural decision violations, legacy architecture
- **Architectural Quality (9.3%)**: Architectural smells, structural dependencies
- **Design Decisions (6.5%)**: Poorly thought decisions, lack of architectural knowledge
- **Maintenance (6.5%)**: Adaptive and corrective maintenance issues

### 2. Code Debt (44.4% of cases)
- **Maintenance (22.4%)**: Corrective (10.2%) and adaptive (7.4%) maintenance
- **Code Quality (13.1%)**: Increased complexity (10.2%), code smells (2.8%)
- **Uncontrolled Changes (9.3%)**: Unmanaged code modifications
- **System Aging (3.7%)**: System size increase, project aging

### 3. Process Debt (20.4% of cases)
- **Knowledge Issues (50% of Process Debt)**: Developer skill deficiencies, domain knowledge gaps
- **Time Pressure (25% of Process Debt)**: Tight deadlines, insufficient time allocation
- **Development Practices (20.8%)**: Fragmented teams, poor agile implementation, inadequate code reviews

## Key Insights

### 1. Socio-Technical Nature
Degradation is **not solely technical** but involves organizational, process, and human factors. Knowledge debt (50% of Process Debt) shows that architecture suffers most when architectural understanding is lost, not just when code breaks.

### 2. Interconnected Debt Ecosystem
The three debt types are **intertwined and reinforce each other**:
- Poor code quality stresses architecture
- Process gaps amplify both architectural and code issues
- Single-layered interventions (like refactoring alone) are insufficient

### 3. Evolution from Reactive to Proactive
The field is shifting from isolated technical detection toward integrated, proactive monitoring that acknowledges interdependence of architecture, code, and team processes.

## Measurement Approaches

### Metrics (54 identified)
- **Architectural Debt (24 metrics)**: Focus on architectural smells (8.3%), coupling (4.2%), cohesion/modularity (2.8% each)
- **Code Debt (30 metrics)**: Emphasis on maintenance metrics (31.9%), code smells (9.7%), growth metrics like Lines of Code (4.2%)

### Measurement Methods
- **Architectural Design (73.1%)**: Reflection models (9%), architecture smell detection (7.5%), consistency analysis (6%)
- **Code Quality (22.4%)**: Anomaly detection (7.5%), stability/evolution metrics (6%)
- **Process Monitoring (4.5%)**: Team and community analysis

## Tools Landscape

### Tool Distribution
- **Architectural Debt Tools (92.1%)**:
  - **Quality Tools**: Arcan (13.2%) for smell detection, Arcade (7.9%) for comprehensive metrics
  - **Design Decision Tools (34.2%)**: Understand, Sonar for structural analysis
  - **Violation Detection (36.8%)**: SonarGraph, JArchitect for conformance checking

- **Code Debt Tools (7.9%)**: Gerrit for code reviews, Declcheck for dependency constraints

### Critical Gap
Most tools focus on **detection rather than remediation**, with limited integration into continuous workflows and poor coverage of socio-technical aspects.

## Remediation Approaches

### Current State
- **Predominantly Reactive**: Focus on fixes like erosion repair after problems manifest
- **Conformance Checking (12%)**: Most significant proactive approach for maintaining architectural alignment
- **Architecture Recovery (6.5%)**: Reactive responses to detected inconsistencies
- **Limited Predictive Capabilities**: Forecasting and awareness methods remain underdeveloped (2.8%)

### Major Limitations
- **Symptom-focused**: Addresses technical symptoms over root causes
- **Fragmented Integration**: Poor connection between detection tools and remediation strategies
- **Process Blindness**: Neglects organizational and process-level interventions

## Critical Findings

### 1. Measurement-Remediation Disconnect
The Sankey plot analysis reveals a **fragmented pipeline**: many studies identify degradation symptoms but fail to translate them into actionable remediation strategies. Most flows end in "No Tool" or "No Remediation" categories.

### 2. Tooling Asymmetry
While architectural design issues are well-instrumented, **Process Debt aspects are poorly supported** by dedicated tools, despite their significant motivational influence.

### 3. Missed Opportunities
Studies frequently identify powerful degradation indicators (architectural smells, traceability gaps, socio-technical issues) but **stop short of proposing actionable remediation strategies** or long-term interventions.

## Future Research Directions

### 1. Holistic Integration
- Develop frameworks linking symptoms to metrics, metrics to tools, and tools to actionable decisions
- Create end-to-end pathways supporting long-term architectural integrity

### 2. Process-Aware Tools
- Build instruments that make process issues visible and actionable
- Address management decisions, developer knowledge, and team structure dimensions

### 3. Proactive Strategies
- Shift from reactive fixes to proactive prevention
- Develop forecasting tools and prioritization heuristics for early intervention

### 4. Continuous Monitoring
- Embed architectural health practices into development workflows
- Integrate degradation detection into CI/CD pipelines

## Conclusions

This comprehensive review reveals that **architectural degradation has evolved from a narrow technical concern to a complex socio-technical phenomenon**. While the field excels at recognizing degradation symptoms through sophisticated metrics and tools, it struggles to translate this knowledge into effective, sustained interventions.

The study's unified definition and debt categorization framework provide a foundation for more holistic approaches. However, the critical need is to **bridge the gap between detection and action**, moving beyond symptom identification toward integrated strategies that address the interconnected nature of architectural, code, and process debt.

The research emphasizes that sustainable architectural health requires not just better tools, but **fundamental changes in how we approach software development as a socio-technical practice**, embedding architectural considerations deeply into organizational processes and team practices.