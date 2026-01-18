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
