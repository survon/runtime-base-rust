---
phase: 6
plan: 1
type: implementation
objective: Implement multi-member council with expertise routing and consensus building
context: Extending existing council service with multi-member support, expertise routing, and consensus building while maintaining all existing patterns and conventions
---

## Objective

Implement Phase 6: Multi-Member Council for the Survon project, extending existing council infrastructure to support multiple council members, expertise routing, and consensus building.

## Context

Building on Phase 1-5 completion, this phase extends the existing council service infrastructure with multi-member support. The existing systems to leverage include:

- ✅ **Council Service** - Phase 4 already implemented
- ✅ **Service Discovery** - Council discovery already implemented  
- ✅ **Module System** - Ready to extend with multi-member strategy
- ✅ **Message Bus** - Event-driven communication ready
- ✅ **LLM Integration** - Phase 3 already implemented

## Tasks

### Task 1: Extend Service Discovery for Multiple Members
**Type:** auto  
**Description:** Extend the existing service discovery system to support multiple council members with expertise categorization.
**Verification:** Service discovery returns multiple council members with categorized expertise.
**Success Criteria:** Discovery system can identify and categorize multiple council members by expertise.

### Task 2: Implement Multi-Member Strategy
**Type:** auto  
**Description:** Create a new multi-member strategy that extends the existing council strategy with multi-member support.
**Verification:** Multi-member strategy correctly handles multiple advisors and routes queries appropriately.
**Success Criteria:** Strategy can manage multiple council members and route queries based on expertise.

### Task 3: Implement Expertise Routing
**Type:** auto  
**Description:** Add expertise routing to the council communication system to route queries to appropriate members.
**Verification:** Queries are routed to council members based on their expertise areas.
**Success Criteria:** Message routing system correctly directs queries to relevant council members.

### Task 4: Implement Consensus Building
**Type:** auto  
**Description:** Add consensus building functionality using existing LLM integration for related queries.
**Verification:** Consensus building works for related queries using LLM analysis.
**Success Criteria:** System can build consensus from multiple council member responses.

### Task 5: Extend UI Framework with Multi-Member Interface
**Type:** auto  
**Description:** Add multi-member interface components to the existing UI framework.
**Verification:** Multi-member interface displays correctly and allows interaction with multiple advisors.
**Success Criteria:** UI shows multiple council members with their expertise and allows query routing.

### Task 6: Extend App Initialization
**Type:** auto  
**Description:** Update app initialization to include multi-member council components.
**Verification:** App starts with multi-member council functionality available.
**Success Criteria:** Multi-member council components are properly initialized on app startup.

### Task 7: Update Main Layout with Multi-Member Interface
**Type:** auto  
**Description:** Add multi-member interface to the main application layout.
**Verification:** Multi-member interface is accessible from main layout.
**Success Criteria:** Users can access and interact with multi-member council from main interface.

## Verification & Success Criteria

### Overall Verification
- Multi-member council system functions correctly
- Expertise routing works as expected
- Consensus building produces coherent results
- UI interface is functional and user-friendly

### Success Criteria
- Multiple council members can be discovered and categorized
- Queries are routed to appropriate members based on expertise
- Consensus building works for related queries
- Multi-member interface is fully functional
- All existing council functionality remains intact

## Output Specification

### Created Files
- `src/module/strategies/multi_member.rs` - New multi-member strategy
- `src/service_council/multi_member/` - New multi-member service components  
- `src/ui/multi_member/` - New multi-member UI components
- `src/service/discovery.rs` - Extended council discovery
- `src/app.rs` - Extended app initialization
- `src/ui/main_layout.rs` - Updated with multi-member interface

### Modified Files
- `src/module/strategies/council.rs` - Extended with multi-member support
- `src/service_council/mod.rs` - Extended with multi-member functionality
- `src/util/io/discovery.rs` - Extended for multiple members
- `src/ui/screens/overview/mod.rs` - Updated to include multi-member screen

### Dependencies
- Extends existing council service infrastructure
- Builds on existing service discovery system
- Leverages existing message bus and LLM integration
- Maintains all existing patterns and conventions