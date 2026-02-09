-- ============================================================================
-- Presave Templates (Reusable, case-independent draft templates)
-- ============================================================================

CREATE TABLE IF NOT EXISTS presave_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID REFERENCES users(id) ON DELETE RESTRICT,

    entity_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT presave_templates_entity_type_valid CHECK (
        entity_type IN ('sender', 'receiver', 'product', 'reporter', 'study', 'narrative')
    )
);

CREATE INDEX idx_presave_templates_org ON presave_templates(organization_id);
CREATE INDEX idx_presave_templates_entity_type ON presave_templates(entity_type);
CREATE INDEX idx_presave_templates_created_by ON presave_templates(created_by);
CREATE INDEX idx_presave_templates_created_at ON presave_templates(created_at DESC);

-- ============================================================================
-- Presave Template Audits (append-only history by template)
-- ============================================================================

CREATE TABLE IF NOT EXISTS presave_template_audits (
    id BIGSERIAL PRIMARY KEY,
    template_id UUID NOT NULL,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    action VARCHAR(50) NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    old_values JSONB,
    new_values JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT presave_template_audits_action_valid CHECK (action IN ('CREATE', 'UPDATE', 'DELETE'))
);

CREATE INDEX idx_presave_template_audits_template_id
    ON presave_template_audits(template_id, created_at DESC);
CREATE INDEX idx_presave_template_audits_org
    ON presave_template_audits(organization_id, created_at DESC);

-- ============================================================================
-- Trigger-based audit capture for presave_templates
-- ============================================================================

CREATE OR REPLACE FUNCTION presave_template_audit_trigger_function()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_user_id UUID;
BEGIN
    v_user_id := get_current_user_context();

    IF TG_OP = 'INSERT' THEN
        INSERT INTO presave_template_audits (
            template_id,
            organization_id,
            action,
            changed_by,
            old_values,
            new_values
        ) VALUES (
            NEW.id,
            NEW.organization_id,
            'CREATE',
            v_user_id,
            NULL,
            to_jsonb(NEW)
        );
        RETURN NEW;

    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO presave_template_audits (
            template_id,
            organization_id,
            action,
            changed_by,
            old_values,
            new_values
        ) VALUES (
            NEW.id,
            NEW.organization_id,
            'UPDATE',
            v_user_id,
            to_jsonb(OLD),
            to_jsonb(NEW)
        );
        RETURN NEW;

    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO presave_template_audits (
            template_id,
            organization_id,
            action,
            changed_by,
            old_values,
            new_values
        ) VALUES (
            OLD.id,
            OLD.organization_id,
            'DELETE',
            v_user_id,
            to_jsonb(OLD),
            NULL
        );
        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$;

CREATE TRIGGER update_presave_templates_updated_at
    BEFORE UPDATE ON presave_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Integrate with shared audit_logs trail as well.
CREATE TRIGGER audit_presave_templates
    AFTER INSERT OR UPDATE OR DELETE ON presave_templates
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_presave_templates_dedicated
    AFTER INSERT OR UPDATE OR DELETE ON presave_templates
    FOR EACH ROW EXECUTE FUNCTION presave_template_audit_trigger_function();

-- ============================================================================
-- Row-Level Security
-- ============================================================================

ALTER TABLE presave_templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE presave_templates FORCE ROW LEVEL SECURITY;

CREATE POLICY presave_templates_org_isolation ON presave_templates
    FOR ALL TO e2br3_app_role
    USING (
        organization_id = current_organization_id() OR is_current_user_admin()
    )
    WITH CHECK (
        organization_id = current_organization_id() OR is_current_user_admin()
    );

ALTER TABLE presave_template_audits ENABLE ROW LEVEL SECURITY;
ALTER TABLE presave_template_audits FORCE ROW LEVEL SECURITY;

CREATE POLICY presave_template_audits_select ON presave_template_audits
    FOR SELECT TO e2br3_app_role
    USING (
        organization_id = current_organization_id() OR is_current_user_admin()
    );

CREATE POLICY presave_template_audits_insert ON presave_template_audits
    FOR INSERT TO e2br3_app_role
    WITH CHECK (
        organization_id = current_organization_id() OR is_current_user_admin()
    );

-- ============================================================================
-- Grants
-- ============================================================================

GRANT SELECT, INSERT, UPDATE, DELETE ON presave_templates TO e2br3_app_role;
GRANT SELECT, INSERT ON presave_template_audits TO e2br3_app_role;
GRANT USAGE, SELECT ON SEQUENCE presave_template_audits_id_seq TO e2br3_app_role;
