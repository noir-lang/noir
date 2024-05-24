output "nlb_arn" {
  value = aws_lb.aztec-network.arn
}

output "nlb_dns" {
  value = aws_lb.aztec-network.dns_name
}

output "p2p_security_group_id" {
  value = aws_security_group.security-group-p2p.id
}

output "p2p_eip" {
  value = aws_eip.aztec_network_p2p_eip.public_ip
}
