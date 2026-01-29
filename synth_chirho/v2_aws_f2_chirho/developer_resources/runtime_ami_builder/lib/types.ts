import {
  MachineImage,
  IMachineImage,
} from 'aws-cdk-lib/aws-ec2';

export interface BaseImageConfig {
  machineImage: IMachineImage;
  user: string;
  description: string;
}

export interface AmiMetadata {
  namePrefix: string;
  description: string;
  tags: Record<string, string>;
}

export const BASE_AMI_CONFIGS = {
  ROCKY_8_10: {
    amiId: "ami-05a3890358c102c97",
    user: "rocky",
    description: "Rocky Linux 8.10"
  },
  UBUNTU_24_04: {
    amiId: "ami-040466f9e4eadfea8",
    user: "ubuntu",
    description: "Ubuntu 24.04 LTS"
  }
} as const;

export enum VivadoLabEditionVersion {
  V2025_1 = '2025.1',
}
