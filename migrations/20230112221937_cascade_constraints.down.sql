-- DOWN

-- Add back plain foreign keys
ALTER TABLE cost_shares 
    DROP CONSTRAINT cost_shares_cost_id_fkey,
    ADD CONSTRAINT cost_shares_cost_id_fkey
        FOREIGN KEY(cost_id)
        REFERENCES costs(id)
        NOT VALID,
    
    DROP CONSTRAINT cost_shares_debtor_id_fkey,
    ADD CONSTRAINT cost_shares_debtor_id_fkey
        FOREIGN KEY(debtor_id)
        REFERENCES users(id)
        NOT VALID;

ALTER TABLE costs
    DROP CONSTRAINT costs_equal_balances_fkey,
    ADD CONSTRAINT costs_equal_balances_fkey
        FOREIGN KEY(equal_balances)
        REFERENCES equal_balances(id)
        NOT VALID,
    
    DROP CONSTRAINT costs_receit_id_fkey,
    ADD CONSTRAINT costs_receit_id_fkey
        FOREIGN KEY(receit_id)
        REFERENCES uploads(id)
        NOT VALID,
    
    DROP CONSTRAINT costs_creditor_id_fkey,
    ADD CONSTRAINT costs_creditor_id_fkey
        FOREIGN KEY(creditor_id)
        REFERENCES users(id)
        NOT VALID,
    
    DROP CONSTRAINT costs_wg_id_fkey,
    ADD CONSTRAINT costs_wg_id_fkey
        FOREIGN KEY(wg_id)
        REFERENCES wgs(id)
        NOT VALID;

ALTER TABLE equal_balances
    DROP CONSTRAINT equal_balances_initiator_id_fkey,
    ADD CONSTRAINT equal_balances_initiator_id_fkey
        FOREIGN KEY(initiator_id)
        REFERENCES users(id)
        NOT VALID,

    DROP CONSTRAINT equal_balances_wg_id_fkey,
    ADD CONSTRAINT equal_balances_wg_id_fkey
        FOREIGN KEY(wg_id)
        REFERENCES wgs(id)
        NOT VALID;

ALTER TABLE users
    DROP CONSTRAINT users_profile_pic_fkey,
    ADD CONSTRAINT users_profile_pic_fkey
        FOREIGN KEY(profile_pic)
        REFERENCES uploads(id)
        NOT VALID;

ALTER TABLE uploads
    DROP CONSTRAINT uploads_access_only_by_wg_fkey,
    ADD CONSTRAINT uploads_access_only_by_wg_fkey
        FOREIGN KEY(access_only_by_wg)
        REFERENCES wgs(id)
        NOT VALID;
    
ALTER TABLE wgs
    DROP CONSTRAINT wgs_profile_pic_fkey,
    ADD CONSTRAINT wgs_profile_pic_fkey
        FOREIGN KEY(profile_pic)
        REFERENCES uploads(id)
        NOT VALID,
    
    DROP CONSTRAINT wgs_header_pic_fkey,
    ADD CONSTRAINT wgs_header_pic_fkey
        FOREIGN KEY(header_pic)
        REFERENCES uploads(id)
        NOT VALID;