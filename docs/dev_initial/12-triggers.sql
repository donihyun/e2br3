CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_cases_updated_at BEFORE UPDATE ON cases
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_patient_info_updated_at BEFORE UPDATE ON patient_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_message_headers_updated_at BEFORE UPDATE ON message_headers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_safety_report_identification_updated_at BEFORE UPDATE ON safety_report_identification
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sender_information_updated_at BEFORE UPDATE ON sender_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_study_information_updated_at BEFORE UPDATE ON study_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_primary_sources_updated_at BEFORE UPDATE ON primary_sources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_patient_death_info_updated_at BEFORE UPDATE ON patient_death_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_parent_information_updated_at BEFORE UPDATE ON parent_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_reactions_updated_at BEFORE UPDATE ON reactions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_test_results_updated_at BEFORE UPDATE ON test_results
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drug_info_updated_at BEFORE UPDATE ON drug_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dosage_information_updated_at BEFORE UPDATE ON dosage_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_narrative_information_updated_at BEFORE UPDATE ON narrative_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Audit Trail Trigger
-- ============================================================================

-- Improved audit trigger function using helper function
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_user_id UUID;
BEGIN
    -- Get user from context (will fail if not set, ensuring user attribution)
    v_user_id := get_current_user_context();

    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, new_values)
        VALUES (TG_TABLE_NAME, NEW.id, 'CREATE', v_user_id, to_jsonb(NEW));
        RETURN NEW;

    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values, new_values)
        VALUES (TG_TABLE_NAME, NEW.id, 'UPDATE', v_user_id, to_jsonb(OLD), to_jsonb(NEW));
        RETURN NEW;

    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values)
        VALUES (TG_TABLE_NAME, OLD.id, 'DELETE', v_user_id, to_jsonb(OLD));
        RETURN OLD;
    END IF;

EXCEPTION
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Audit trail logging failed for table %.%: %. User context may not be set.',
            TG_TABLE_SCHEMA, TG_TABLE_NAME, SQLERRM;
END;
$$;

-- Audit trigger for tables that use audit_id UUID instead of id UUID.
CREATE OR REPLACE FUNCTION audit_trigger_function_with_audit_id()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_user_id UUID;
BEGIN
    v_user_id := get_current_user_context();

    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, new_values)
        VALUES (TG_TABLE_NAME, NEW.audit_id, 'CREATE', v_user_id, to_jsonb(NEW));
        RETURN NEW;

    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values, new_values)
        VALUES (TG_TABLE_NAME, NEW.audit_id, 'UPDATE', v_user_id, to_jsonb(OLD), to_jsonb(NEW));
        RETURN NEW;

    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values)
        VALUES (TG_TABLE_NAME, OLD.audit_id, 'DELETE', v_user_id, to_jsonb(OLD));
        RETURN OLD;
    END IF;

EXCEPTION
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Audit trail logging failed for table %.%: %. User context may not be set.',
            TG_TABLE_SCHEMA, TG_TABLE_NAME, SQLERRM;
END;
$$;

CREATE TRIGGER audit_cases AFTER INSERT OR UPDATE OR DELETE ON cases
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_patient_info AFTER INSERT OR UPDATE OR DELETE ON patient_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_message_headers AFTER INSERT OR UPDATE OR DELETE ON message_headers
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_safety_report_identification AFTER INSERT OR UPDATE OR DELETE ON safety_report_identification
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_sender_information AFTER INSERT OR UPDATE OR DELETE ON sender_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_literature_references AFTER INSERT OR UPDATE OR DELETE ON literature_references
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_study_information AFTER INSERT OR UPDATE OR DELETE ON study_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_study_registration_numbers AFTER INSERT OR UPDATE OR DELETE ON study_registration_numbers
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_primary_sources AFTER INSERT OR UPDATE OR DELETE ON primary_sources
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_medical_history_episodes AFTER INSERT OR UPDATE OR DELETE ON medical_history_episodes
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_past_drug_history AFTER INSERT OR UPDATE OR DELETE ON past_drug_history
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_patient_death_information AFTER INSERT OR UPDATE OR DELETE ON patient_death_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_reported_causes_of_death AFTER INSERT OR UPDATE OR DELETE ON reported_causes_of_death
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_autopsy_causes_of_death AFTER INSERT OR UPDATE OR DELETE ON autopsy_causes_of_death
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_parent_information AFTER INSERT OR UPDATE OR DELETE ON parent_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_reactions AFTER INSERT OR UPDATE OR DELETE ON reactions
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_test_results AFTER INSERT OR UPDATE OR DELETE ON test_results
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_drug_information AFTER INSERT OR UPDATE OR DELETE ON drug_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_drug_active_substances AFTER INSERT OR UPDATE OR DELETE ON drug_active_substances
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_dosage_information AFTER INSERT OR UPDATE OR DELETE ON dosage_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_drug_indications AFTER INSERT OR UPDATE OR DELETE ON drug_indications
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_narrative_information AFTER INSERT OR UPDATE OR DELETE ON narrative_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_sender_diagnoses AFTER INSERT OR UPDATE OR DELETE ON sender_diagnoses
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_case_summary_information AFTER INSERT OR UPDATE OR DELETE ON case_summary_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_case_versions AFTER INSERT OR UPDATE OR DELETE ON case_versions
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Audit triggers for core tables
CREATE TRIGGER audit_organizations AFTER INSERT OR UPDATE OR DELETE ON organizations
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_users AFTER INSERT OR UPDATE OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Phase 1-3 new tables audit triggers
CREATE TRIGGER audit_receiver_information AFTER INSERT OR UPDATE OR DELETE ON receiver_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_other_case_identifiers AFTER INSERT OR UPDATE OR DELETE ON other_case_identifiers
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_linked_report_numbers AFTER INSERT OR UPDATE OR DELETE ON linked_report_numbers
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_parent_medical_history AFTER INSERT OR UPDATE OR DELETE ON parent_medical_history
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_parent_past_drug_history AFTER INSERT OR UPDATE OR DELETE ON parent_past_drug_history
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_drug_recurrence_information AFTER INSERT OR UPDATE OR DELETE ON drug_recurrence_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_drug_reaction_assessments AFTER INSERT OR UPDATE OR DELETE ON drug_reaction_assessments
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_relatedness_assessments AFTER INSERT OR UPDATE OR DELETE ON relatedness_assessments
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Terminology tables audit triggers
CREATE TRIGGER audit_meddra_terms AFTER INSERT OR UPDATE OR DELETE ON meddra_terms
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function_with_audit_id();

CREATE TRIGGER audit_whodrug_products AFTER INSERT OR UPDATE OR DELETE ON whodrug_products
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function_with_audit_id();

CREATE TRIGGER audit_iso_countries AFTER INSERT OR UPDATE OR DELETE ON iso_countries
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function_with_audit_id();

CREATE TRIGGER audit_e2b_code_lists AFTER INSERT OR UPDATE OR DELETE ON e2b_code_lists
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function_with_audit_id();

-- ============================================================================
-- Additional updated_at triggers for new tables
-- ============================================================================

CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_literature_references_updated_at BEFORE UPDATE ON literature_references
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_study_registration_numbers_updated_at BEFORE UPDATE ON study_registration_numbers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_medical_history_episodes_updated_at BEFORE UPDATE ON medical_history_episodes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_past_drug_history_updated_at BEFORE UPDATE ON past_drug_history
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_reported_causes_of_death_updated_at BEFORE UPDATE ON reported_causes_of_death
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_autopsy_causes_of_death_updated_at BEFORE UPDATE ON autopsy_causes_of_death
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drug_active_substances_updated_at BEFORE UPDATE ON drug_active_substances
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drug_indications_updated_at BEFORE UPDATE ON drug_indications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sender_diagnoses_updated_at BEFORE UPDATE ON sender_diagnoses
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_case_summary_information_updated_at BEFORE UPDATE ON case_summary_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Phase 1-3 new tables updated_at triggers
CREATE TRIGGER update_receiver_information_updated_at BEFORE UPDATE ON receiver_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_other_case_identifiers_updated_at BEFORE UPDATE ON other_case_identifiers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_linked_report_numbers_updated_at BEFORE UPDATE ON linked_report_numbers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_parent_medical_history_updated_at BEFORE UPDATE ON parent_medical_history
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_parent_past_drug_history_updated_at BEFORE UPDATE ON parent_past_drug_history
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drug_recurrence_information_updated_at BEFORE UPDATE ON drug_recurrence_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drug_reaction_assessments_updated_at BEFORE UPDATE ON drug_reaction_assessments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_relatedness_assessments_updated_at BEFORE UPDATE ON relatedness_assessments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Row-Level Security for Organization Isolation (Case-Related Tables)
-- ============================================================================
-- Note: Core table RLS (cases, users, organizations) is in 03-safetydb-schema.sql
-- This section adds RLS for all case-related nested tables

-- Patient Information
ALTER TABLE patient_information ENABLE ROW LEVEL SECURITY;
CREATE POLICY patient_info_via_case ON patient_information
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = patient_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = patient_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Drug Information
ALTER TABLE drug_information ENABLE ROW LEVEL SECURITY;
CREATE POLICY drug_info_via_case ON drug_information
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = drug_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = drug_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Reactions
ALTER TABLE reactions ENABLE ROW LEVEL SECURITY;
CREATE POLICY reactions_via_case ON reactions
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = reactions.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = reactions.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Test Results
ALTER TABLE test_results ENABLE ROW LEVEL SECURITY;
CREATE POLICY test_results_via_case ON test_results
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = test_results.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = test_results.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Message Headers
ALTER TABLE message_headers ENABLE ROW LEVEL SECURITY;
CREATE POLICY message_headers_via_case ON message_headers
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = message_headers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = message_headers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Safety Report Identification
ALTER TABLE safety_report_identification ENABLE ROW LEVEL SECURITY;
CREATE POLICY safety_report_id_via_case ON safety_report_identification
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = safety_report_identification.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = safety_report_identification.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Narrative Information
ALTER TABLE narrative_information ENABLE ROW LEVEL SECURITY;
CREATE POLICY narrative_info_via_case ON narrative_information
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = narrative_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = narrative_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Drug Reaction Assessments (via drug_information)
ALTER TABLE drug_reaction_assessments ENABLE ROW LEVEL SECURITY;
CREATE POLICY drug_reaction_assessments_via_case ON drug_reaction_assessments
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM drug_information di
            JOIN cases c ON c.id = di.case_id
            WHERE di.id = drug_reaction_assessments.drug_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM drug_information di
            JOIN cases c ON c.id = di.case_id
            WHERE di.id = drug_reaction_assessments.drug_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Other Case Identifiers
ALTER TABLE other_case_identifiers ENABLE ROW LEVEL SECURITY;
CREATE POLICY other_case_ids_via_case ON other_case_identifiers
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = other_case_identifiers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = other_case_identifiers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Linked Report Numbers
ALTER TABLE linked_report_numbers ENABLE ROW LEVEL SECURITY;
CREATE POLICY linked_reports_via_case ON linked_report_numbers
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = linked_report_numbers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = linked_report_numbers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Primary Sources
ALTER TABLE primary_sources ENABLE ROW LEVEL SECURITY;
CREATE POLICY primary_sources_via_case ON primary_sources
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = primary_sources.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = primary_sources.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Literature References
ALTER TABLE literature_references ENABLE ROW LEVEL SECURITY;
CREATE POLICY literature_refs_via_case ON literature_references
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = literature_references.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = literature_references.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Study Information
ALTER TABLE study_information ENABLE ROW LEVEL SECURITY;
CREATE POLICY study_info_via_case ON study_information
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = study_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = study_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Sender Information
ALTER TABLE sender_information ENABLE ROW LEVEL SECURITY;
CREATE POLICY sender_info_via_case ON sender_information
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = sender_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = sender_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Receiver Information
ALTER TABLE receiver_information ENABLE ROW LEVEL SECURITY;
CREATE POLICY receiver_info_via_message ON receiver_information
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = receiver_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = receiver_information.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );
