-- USER DELETION::
 --By the end of the transaction, this row to be gone or user deletion can't proceed
--User should not be deleted for this reason, only renamed and stripped of personal info
-- hence the "no action"
-- RESTRICT would immediately raise error

-- WG DELETION::
-- The Wg, the collective, is the actual owner of the financial data.
-- If it dies, all data gets burnt and users kicked out.
-- hence "CASCADE"
-- (if there are confidential uploads left, referencing wgs with access_only_by_wg)
-- (those need to be removed by the application before deletion!!)

-- ATTATCHMENT VANISHING::
--blocking upload deletion might tangle things: 
--in principle upload/obj should be 1:1, but since it's not enforced...
 -- just set null to avoid mess, attatchment vanishes

ALTER TABLE cost_shares 
    DROP CONSTRAINT cost_shares_cost_id_fkey,
    ADD CONSTRAINT cost_shares_cost_id_fkey
        FOREIGN KEY(cost_id)
        REFERENCES costs(id)
        ON UPDATE no action
        -- a share belongs solely to the cost, and lives and dies with it.
        ON DELETE CASCADE
        NOT VALID,
    
    DROP CONSTRAINT cost_shares_debtor_id_fkey,
    ADD CONSTRAINT cost_shares_debtor_id_fkey
        FOREIGN KEY(debtor_id)
        REFERENCES users(id)
        ON UPDATE no action
        -- see USER DELETION
        ON DELETE no action 

        NOT VALID;

ALTER TABLE costs
    DROP CONSTRAINT costs_equal_balances_fkey,
    ADD CONSTRAINT costs_equal_balances_fkey
        FOREIGN KEY(equal_balances)
        REFERENCES equal_balances(id)
        ON UPDATE no action
        --only the most recent balance should be deleted, carefully by the application! (or not at all tbh)
        ON DELETE SET NULL -- dumps costs back into main feed!!
        NOT VALID,
    
    DROP CONSTRAINT costs_receit_id_fkey,
    ADD CONSTRAINT costs_receit_id_fkey
        FOREIGN KEY(receit_id)
        REFERENCES uploads(id)
        ON UPDATE no action
        -- see ATTATCHMENT VANISHING
        ON DELETE SET NULL
        NOT VALID,
    
    DROP CONSTRAINT costs_creditor_id_fkey,
    ADD CONSTRAINT costs_creditor_id_fkey
        FOREIGN KEY(creditor_id)
        REFERENCES users(id)
        ON UPDATE no action
        -- see USER DELETION
        ON DELETE no action
        NOT VALID,
    
    DROP CONSTRAINT costs_wg_id_fkey,
    ADD CONSTRAINT costs_wg_id_fkey
        FOREIGN KEY(wg_id)
        REFERENCES wgs(id)
        ON UPDATE no action
        -- see WG DELETION
        ON DELETE CASCADE
        NOT VALID;

ALTER TABLE equal_balances
    DROP CONSTRAINT equal_balances_initiator_id_fkey,
    ADD CONSTRAINT equal_balances_initiator_id_fkey
        FOREIGN KEY(initiator_id)
        REFERENCES users(id)
        ON UPDATE no action
        -- see USER DELETION
        ON DELETE no action
        NOT VALID,

    DROP CONSTRAINT equal_balances_wg_id_fkey,
    ADD CONSTRAINT equal_balances_wg_id_fkey
        FOREIGN KEY(wg_id)
        REFERENCES wgs(id)
        ON UPDATE no action
        -- see WG DELETION
        ON DELETE CASCADE
        NOT VALID;

ALTER TABLE users
    DROP CONSTRAINT users_profile_pic_fkey,
    ADD CONSTRAINT users_profile_pic_fkey
        FOREIGN KEY(profile_pic)
        REFERENCES uploads(id)
        ON UPDATE no action
        -- see ATTATCHMENT VANISHING
        ON DELETE SET NULL
        NOT VALID;

ALTER TABLE uploads
    DROP CONSTRAINT uploads_access_only_by_wg_fkey,
    ADD CONSTRAINT uploads_access_only_by_wg_fkey
        FOREIGN KEY(access_only_by_wg)
        REFERENCES wgs(id)
        ON UPDATE no action
        -- Trying to delete a wg before having removed it's confidential files?
        -- Bozo!!
        ON DELETE no action
        NOT VALID;
    
ALTER TABLE wgs
    DROP CONSTRAINT wgs_profile_pic_fkey,
    ADD CONSTRAINT wgs_profile_pic_fkey
        FOREIGN KEY(profile_pic)
        REFERENCES uploads(id)
        ON UPDATE no action
        -- see ATTATCHMENT VANISHING
        ON DELETE SET NULL
        NOT VALID,
    
    DROP CONSTRAINT wgs_header_pic_fkey,
    ADD CONSTRAINT wgs_header_pic_fkey
        FOREIGN KEY(header_pic)
        REFERENCES uploads(id)
        ON UPDATE no action
        -- see ATTATCHMENT VANISHING
        ON DELETE SET NULL
        NOT VALID;