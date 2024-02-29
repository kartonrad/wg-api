SELECT  debtor_id, creditor_id, (amount/nr_shares)::NUMERIC(16,2) as owed, paid, id as cost_id , wg_id, equal_balances
FROM cost_shares
LEFT JOIN (
	SELECT
		costs.id,
		amount,
		creditor_id,
		wg_id,
		equal_balances,
		count(*) as nr_shares,
		sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares
	FROM costs
	LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id   --multiple per row
	GROUP BY costs.id
) AS cost_agg ON cost_agg.id = cost_shares.cost_id



--- for GET /costs
SELECT id, wg_id, name, amount, creditor_id, receit_id, added_on, equal_balances, ROW(my_share.cost_id, my_share.debtor_id, my_share.paid) as my_share,
	count(*) as nr_shares, sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares
FROM costs
LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
LEFT JOIN cost_shares as my_share ON costs.id = my_share.cost_id AND my_share.debtor_id = 1 -- guarranteed to be unique per row, as (cost_id, debtor_id) is PRIMARY
WHERE wg_id = 1 AND coalesce(equal_balances, 0) = 0
GROUP BY costs.id, my_share.cost_id, my_share.debtor_id, my_share.paid;


WITH debt_table AS (
	SELECT debtor_id, creditor_id, (amount/nr_shares)::NUMERIC(16,2) as owed
	FROM cost_shares
	LEFT JOIN (
		SELECT costs.id, amount, creditor_id, wg_id, equal_balances,
			count(*) as nr_shares, sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares
		FROM costs
		LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
		GROUP BY costs.id
	) AS cost_agg ON cost_agg.id = cost_shares.cost_id
	WHERE debtor_id != creditor_id AND paid = false AND cost_agg.wg_id = 1 AND coalesce(equal_balances, 0) = 2
), recieve_table AS (
	SELECT creditor_id as user_id, sum(owed) as to_recieve
	FROM debt_table
	GROUP BY creditor_id
), pay_table AS (
	SELECT debtor_id as user_id, sum(owed) as to_pay
	FROM debt_table
	GROUP BY debtor_id
)
SELECT * FROM recieve_table
FULL OUTER JOIN pay_table ON( recieve_table.user_id = pay_table.user_id );
	

SELECT sum(1) FROM costs WHERE wg_id = 17


SELECT equal_balances.id, equal_balances.balanced_on, equal_balances.initiator_id, equal_balances.wg_id, 
	coalesce( sum(costs.amount), 0) as total_unified_spending,
	coalesce( sum( CASE WHEN costs.paid = false AND costs.debtor_id != costs.creditor_id THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_paid,
	coalesce( sum( CASE WHEN creditor_id = 2 THEN (costs.amount/costs.nr_shares*costs.nr_unpaid_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_recieved,
	coalesce( sum( CASE WHEN costs.paid IS NOT NULL THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) AS my_total_spending
FROM equal_balances
LEFT JOIN (
	SELECT id, amount, creditor_id, added_on, equal_balances, my_share.paid, my_share.debtor_id,
		count(*) as nr_shares, coalesce( sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END) , 0) as nr_unpaid_shares
	FROM costs
	LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
	LEFT JOIN cost_shares as my_share ON costs.id = my_share.cost_id AND my_share.debtor_id = 2 -- guarranteed to be unique per row, as (cost_id, debtor_id) is PRIMARY
	WHERE wg_id = 1
	GROUP BY costs.id, my_share.cost_id, my_share.paid, my_share.debtor_id
) AS costs ON costs.equal_balances = equal_balances.id
WHERE wg_id = 1
GROUP BY equal_balances.id, equal_balances.balanced_on, equal_balances.initiator_id, equal_balances.wg_id;


SELECT
	date_trunc('week', added_on) as time_bucket ,
	coalesce( sum(costs.amount), 0) as total_unified_spending,
	coalesce( sum( CASE WHEN costs.paid = false AND costs.debtor_id != costs.creditor_id THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_paid,
	coalesce( sum( CASE WHEN creditor_id = 2 THEN (costs.amount/costs.nr_shares*costs.nr_unpaid_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_recieved,
	coalesce( sum( CASE WHEN costs.paid IS NOT NULL THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) AS my_total_spending
FROM (
	SELECT id, amount, creditor_id, added_on, equal_balances, my_share.paid, my_share.debtor_id,
		count(*) as nr_shares, coalesce( sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END) , 0) as nr_unpaid_shares
	FROM costs
	LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
	LEFT JOIN cost_shares as my_share ON costs.id = my_share.cost_id AND my_share.debtor_id = 2 -- guarranteed to be unique per row, as (cost_id, debtor_id) is PRIMARY
	WHERE wg_id = 1
	GROUP BY costs.id, my_share.cost_id, my_share.paid, my_share.debtor_id
) AS costs
GROUP BY time_bucket;