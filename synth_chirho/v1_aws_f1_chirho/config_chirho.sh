# For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
# AWS F1 Configuration ☧
# Generated on 2026-01-26

export AWS_REGION_CHIRHO="us-east-1"
export S3_BUCKET_CHIRHO="minikanren-fpga-chirho-686672719245"
export KEY_NAME_CHIRHO="minikanren-fpga-key-chirho"
export SECURITY_GROUP_CHIRHO="sg-0b29ce11e8f0878cd"
export VPC_ID_CHIRHO="vpc-c7848ea1"
export SUBNET_ID_CHIRHO="subnet-c5104ea0"  # us-east-1a, public subnet

# AMI IDs
export FPGA_DEV_AMI_CHIRHO="ami-01198b89d80ebfdd2"  # FPGA Developer AMI 1.17.0 Ubuntu

# Instance types
export SYNTH_INSTANCE_CHIRHO="c5.18xlarge"  # For Vivado synthesis (~$3.06/hr) - 72 vCPUs, 144GB RAM
export F1_INSTANCE_CHIRHO="f1.2xlarge"     # For FPGA execution (~$1.65/hr)
