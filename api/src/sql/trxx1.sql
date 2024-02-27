-- debtor_id != creditor_id should be in CHECK constraint of transactions

SELECT debtor_id, creditor_id, (amount/nr_shares)::NUMERIC(16,2) as owed, added_on
FROM cost_shares
LEFT JOIN (
	SELECT costs.id, amount, creditor_id, wg_id, equal_balances, added_on,
		count(*) as nr_shares, sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares
	FROM costs
	LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id   --multiple per row
	GROUP BY costs.id
) AS cost_agg ON cost_agg.id = cost_shares.cost_id
WHERE debtor_id != creditor_id AND paid = false AND cost_agg.wg_id = 1 AND coalesce(equal_balances, 0) = 0
UNION
SELECT debtor_id, creditor_id, amount as owed, added_on FROM transactions
WHERE wg_id=1 AND coalesce(equal_balances, 0) = 0
ORDER BY added_on DESC;


WITH debt_table AS (        
	SELECT debtor_id, creditor_id, (amount/nr_shares)::NUMERIC(16,2) as owed, added_on
	FROM cost_shares
	LEFT JOIN (
		SELECT costs.id, amount, creditor_id, wg_id, equal_balances, added_on,
			count(*) as nr_shares, sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares
		FROM costs
		LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id   --multiple per row
		GROUP BY costs.id
	) AS cost_agg ON cost_agg.id = cost_shares.cost_id
	WHERE debtor_id != creditor_id AND paid = false AND cost_agg.wg_id = 1 AND coalesce(equal_balances, 0) = 0
	UNION
	SELECT debtor_id, creditor_id, amount as owed, added_on FROM transactions
	WHERE wg_id=1 AND coalesce(equal_balances, 0) = 0
	ORDER BY added_on DESC
), recieve_table AS (                                                   
	SELECT creditor_id as user_id, sum(owed) as to_recieve
	FROM debt_table
	GROUP BY creditor_id
), pay_table AS (
	SELECT debtor_id as user_id, sum(owed) as to_pay
	FROM debt_table
	GROUP BY debtor_id
)
SELECT recieve_table.user_id as u1, to_recieve, pay_table.user_id as u2, to_pay FROM recieve_table
FULL OUTER JOIN pay_table ON( recieve_table.user_id = pay_table.user_id );