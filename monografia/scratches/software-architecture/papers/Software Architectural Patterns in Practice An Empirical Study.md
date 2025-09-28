> This paper states about how patterns are suboptimaly choosen
> My work is about evaluating the fitting of Actor Model outside it's designated domain
### Gap Between Theory and Practice

"Surprisingly, the results from the survey reflected that; in practice, the quality requirements are not the dominant factor when deciding on which patterns to select. In fact, it is the functionality that is taking the front seat."

### Need for Practice Improvement

"The implication from these findings is that there is a need to improve the current practice when selecting software architectural patterns to include quality attributes as a selection criteria."

### Architecture's Fundamental Purpose

"In fact, if functionality was the only thing that mattered, we would not have to divide the system into pieces at all, and we would not need to have an architecture."
## Conclusions and Recommendations

### Primary Recommendations from the Paper

The paper provides specific suggestions for addressing the identified gaps between theory and practice:

#### 1. **Improve Pattern Selection Practices**

**What the paper suggests:**
- **Include quality attributes as primary selection criteria** rather than focusing predominantly on functionality
- Move away from the current "short-sighted practice" of functionality-first selection
- Adopt a more systematic approach that considers quality requirements early in the pattern selection process

**Supporting Evidence:**
> "This indicates a need to improve the current short-sighted practice when selecting architectural patterns to consider qualities as criteria of selection more often."

The paper emphasizes that teams who considered quality attributes reported 62% satisfaction with pattern effectiveness versus only 43% for those who didn't consider quality.

#### 2. **Better Requirements Management Integration**

**What the paper suggests:**
- **Improve harmonization between requirements management activities and software architecture processes**
- Address the root cause of the most significant challenge: continuous changes in requirements/environment

**Specific Quote:**
> "This finding suggests a better harmonization of the requirements management activities and software architecture process."

**Context:** Since ~50% of participants identified continuous requirement changes as their main challenge, the paper suggests this integration as a critical improvement area.

#### 3. **Quality-Driven Architecture Approach**

**What the paper suggests:**
- **Align industry practice with architectural theory** that emphasizes quality attribute satisfaction as the primary purpose of architecture
- Recognize that architecture's fundamental value lies in addressing quality concerns, not just functionality

**Theoretical Foundation:**
> "The literature on software architecture suggests that one of the primary purposes of the architecture of a system is to create a system design to satisfy the quality attributes."

#### 4. **Research and Industry Collaboration**

**What the paper suggests:**
- **Conduct replication studies** to strengthen the findings
- **Further investigate specific anomalies** (like security's low prioritization in critical domains)
- **Develop tools and methods** to support quality-driven pattern selection

**Future Work Statement:**
> "We hope that this survey and corresponding results stimulate research into prevailing software practices. Moreover, we intend these results to highlight the areas of software architecture, software quality management, and software project management that need the attention of both the research community and the industry professional."

#### 5. **Educational and Training Implications**

**Implicit Suggestions:**
While not explicitly stated as recommendations, the paper's findings suggest:
- **Training programs** should emphasize quality-driven pattern selection
- **Architecture education** should bridge the gap between theoretical knowledge and practical application
- **Industry guidelines** should be developed to promote better pattern selection practices

#### 6. **Organizational Process Improvements**

**What the paper implies:**
- Organizations should **evaluate their current pattern selection processes**
- **Establish criteria frameworks** that prioritize quality attributes alongside functionality
- **Implement feedback mechanisms** to assess pattern effectiveness in achieving project goals

### Key Limitation of Recommendations

The paper acknowledges that while it identifies problems and suggests directions for improvement, it doesn't provide **detailed methodological frameworks** for implementing these recommendations. The authors state:

> "We plan also in the future to perform a replication of the study to increase the solidity of our findings."

This suggests the recommendations are foundational insights that require further research to develop into actionable methodologies and tools for industry adoption.

### Industry Impact

The study provides empirical evidence that current industry practices may be suboptimal, potentially leading to architectural decisions that don't fully leverage patterns' quality-enhancing capabilities. This suggests significant room for improvement in how organizations approach architectural pattern selection and implementation.

## Key Definitions from the Paper

### **Software Architecture**

The paper provides multiple perspectives on software architecture:

**Primary Definition:**
> "A software architecture is significant in paving the way for software success for many reasons. It is a structure that facilitates the satisfaction to systemic qualities (such as performance, security, availability, modifiability, etc.)."

**Functional Aspects:**
- **Communication Vehicle:** "serves as a vehicle for communication to the stakeholders of the system under consideration"
- **Reuse Foundation:** "can serve as a basis for large-scale reuse"
- **Traceability Bridge:** "enhances the traceability between the requirements and the technical solutions which reduce risks associated with building the technical solution"

**Decision-Making Framework:**
> "Defining Software Architecture involves a series of decisions based on many factors in a wide range of software development. These decisions are analogous to load-bearing walls of a building. Once put in place, altering them is extremely difficult and expensive."

**Quality-Centric Purpose:**
> "In fact, if functionality was the only thing that mattered, we would not have to divide the system into pieces at all, and we would not need to have an architecture. The architecture is the first place in software creation in which quality requirements should be addressed."

### **Software Architectural Patterns**

**Core Definition:**
> "A software architectural pattern defines a family of systems in terms of a pattern of structural organization and behavior."

**Technical Specification:**
> "More specifically, an architectural pattern determines the vocabulary of components and connectors that can be used in instances of that pattern, together with a set of constraints on how they can be combined."

**Purpose and Function:**
> "Architects might face recurring issues in different software architecture design. For saving of huge cost and the reduction of risks, software architecture decisions can rely on a set of idiomatic patterns commonly named architectural styles or patterns."

**Common Framework Elements:**
The paper notes patterns are described using frameworks that include:
- **Name** - Pattern identifier
- **Problem** - The recurring issue addressed
- **Structure and Dynamics** - The solution organization
- **Consequences** - Benefits and liabilities of using the pattern

**Examples Mentioned:**
- Layers, Pipes-Filters, Model View Controller (MVC), Broker, Client-Server

### **Architectural Tactics**

**Definition:**
> "Tactics are a special type of operationalization that serves as the meeting point between the quality attributes and the software architecture."

**Technical Description:**
> "An architectural tactic as an architectural transformation that affects the parameters of an underlying quality attribute model."

**Relationship to Patterns:**
> "The structure and behavior of tactics is more local and low level than the architectural pattern and therefore must fit into the larger structure and behavior of patterns applied to the same system."

**Implementation Impact:**
> "Implementing a certain tactic within a pattern may affect the pattern by modifying some of its components, adding some components and connectors, or replicating components and connectors."

### **Quality Attributes**

**Definition:**
> "Quality is the totality of characteristics of an entity that bear on its ability to satisfy stated and implied needs. Software quality is an essential and distinguishing attribute of the final product."

**Architectural Significance:**
> "Typically, systems have multiple important quality attributes (e.g., Performance, Modifiability, Security, Testability, etc.) and decisions made to satisfy a particular quality may help or hinder the achievement of another quality attribute."

**Examples Provided:**
- Performance
- Security  
- Availability
- Modifiability
- Testability
- Usability
- Interoperability

### **Components and Connectors**

While not explicitly defined, the paper refers to these as the **"vocabulary"** elements that patterns determine:

**Components:** The structural elements of the system (implied as the functional units)

**Connectors:** The interaction mechanisms between components (implied as the communication/integration elements)

### **Architectural Drivers**

**Definition (implied):**
> "The motivation for the selection of the appropriate patterns should depend mostly on a set of architectural drivers (mostly quality requirements) which possess the highest business and technical priority to the system."

### **System Structure**

**Hierarchical Concept:**
> "The architecture of a software system is almost never limited to a single architectural pattern. Complex systems exhibit multiple patterns at once. A web-based system might employ a three-tier client–server pattern, but within this pattern it might also use MVC, layering and so forth."

### **Architectural Styles**

**Relationship to Patterns:**
The paper uses "architectural styles" and "architectural patterns" **interchangeably**, referring to the same concept:
> "software architecture decisions can rely on a set of idiomatic patterns commonly named architectural styles or patterns"

## Key Conceptual Relationships

The paper establishes these important relationships:

1. **Architecture → Patterns → Tactics**: Architecture uses patterns, which can be refined with tactics
2. **Patterns → Quality Attributes**: Patterns address multiple quality attributes simultaneously  
3. **Quality Attributes → Architectural Decisions**: Quality requirements should drive pattern selection
4. **Components + Connectors + Constraints = Pattern**: The essential elements that define a pattern

These definitions collectively present software architecture as a **quality-driven, decision-making framework** where patterns serve as proven solutions to recurring design problems, implemented through components and connectors, and refined through tactics to achieve specific quality goals.