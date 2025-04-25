use rand::Rng;
use secp256k1::{Secp256k1, PublicKey, SecretKey};
use sha2::{Sha256, Digest};
use bs58;
use hex;

#[derive(Clone, Debug)]
pub struct Wallet {
    private_key: SecretKey,
    public_key: PublicKey,
    address: String,
}

impl Wallet {
    pub fn new() -> Self {
        // 1. Özel anahtar oluştur (256 bit rastgele sayı)
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        // 32 byte'lık rastgele bir sayı oluşturup SecretKey'e dönüştür
        let random_bytes: [u8; 32] = core::array::from_fn(|_| rng.gen());
        let secret_key = SecretKey::from_slice(&random_bytes).expect("32 bytes secret key");
        
        // 2. Genel anahtarı elde et (ECDSA kullanılır)
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        // 3-6. Adres oluştur
        let address = Self::generate_address(&public_key);
        
        Wallet {
            private_key: secret_key,
            public_key,
            address,
        }
    }
    
    pub fn get_address(&self) -> &str {
        &self.address
    }
    
    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }
    
    fn generate_address(public_key: &PublicKey) -> String {
        // Public key'i hash'le (Bitcoin'den farklı olarak sadece tek bir SHA-256 kullan)
        let public_key_bytes = public_key.serialize();
        
        // SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&public_key_bytes);
        let hash_result = hasher.finalize();
        
        // Version byte ekle (0x00)
        let mut versioned_hash = vec![0x00];
        versioned_hash.extend_from_slice(&hash_result[..20]); // İlk 20 byte'ı al (RIPEMD160 gibi 20 byte'a sıkıştırılmış)
        
        // Checksum ekle (verinin SHA-256'sının SHA-256'sından ilk 4 byte)
        let mut checksum_hasher1 = Sha256::new();
        checksum_hasher1.update(&versioned_hash);
        let checksum_result1 = checksum_hasher1.finalize();
        
        let mut checksum_hasher2 = Sha256::new();
        checksum_hasher2.update(&checksum_result1);
        let checksum_result2 = checksum_hasher2.finalize();
        
        // İlk 4 byte'ı al
        let checksum = &checksum_result2[0..4];
        
        // Versiyonlu hash ile checksum'ı birleştir
        let mut address_bytes = versioned_hash.clone();
        address_bytes.extend_from_slice(checksum);
        
        // Base58Check kodlaması yap
        bs58::encode(address_bytes).into_string()
    }
    
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let secp = Secp256k1::new();
        
        // İlk olarak verinin hash'ini al
        let mut hasher = Sha256::new();
        hasher.update(data);
        let message_hash = hasher.finalize();
        
        // Hash'i bir message tipine dönüştür
        let message = secp256k1::Message::from_digest_slice(&message_hash).expect("32 bytes");
        
        // İmzala
        let signature = secp.sign_ecdsa(&message, &self.private_key);
        
        // İmzayı byte dizisine dönüştür
        signature.serialize_der().to_vec()
    }
    
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        let secp = Secp256k1::new();
        
        // İlk olarak verinin hash'ini al
        let mut hasher = Sha256::new();
        hasher.update(data);
        let message_hash = hasher.finalize();
        
        // Hash'i bir message tipine dönüştür
        let message = secp256k1::Message::from_digest_slice(&message_hash).expect("32 bytes");
        
        // İmzayı doğrula
        let signature = secp256k1::ecdsa::Signature::from_der(signature).expect("Geçerli imza");
        
        secp.verify_ecdsa(&message, &signature, &self.public_key).is_ok()
    }
} 