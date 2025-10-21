#![no_std]  // Set up inicial
use soroban_sdk::{
    contract, contractimpl, contracterror, contracttype, log, // definir errores, DataKey
    Env, Symbol, Address, String  // ⭐ String para validar inputs de texto, control de acceso
  
};

//definición de errores
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NombreVacio = 1,
    NombreMuyLargo = 2,
    NoAutorizado = 3, // error si alguien intenta resetear contador sin ser adm
    NoInicializado = 4,
}

// definición de DataKey
#[contracttype]
#[derive(Clone)]
pub enum DataKey { // tipo de storage??
    Admin, // sin parametros
    ContadorSaludos,
    UltimoSaludo(Address), // con parametro
}

//definición del contrato
#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    // implementación de initialize()
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> { // tiene manejo de errores por tipo
        
        // verificar si está inicializado
        if env.storage().instance().has(&DataKey::Admin) { // has vs get. has mas barato, solo verifica existencia son deserializar
            return Err(Error::NoInicializado);
        }

        // guardar el admin luego de verificar que este inicializado
        env.storage().instance().set(&DataKey::Admin, &admin); // instanceStorage. porq?

        // inicializar contador=0, tipo de dato u32. 
        env.storage().instance().set(&DataKey::ContadorSaludos, &0u32);

        // extender TTL
        env.storage().instance().extend_ttl(100, 100); // qie significan los 100
    
        Ok(())

    }
    // implementación de hello() // ⭐ String ya tiene .len(), no necesitamos .to_string()
    pub fn hello(env: Env,usuario: Address,nombre: String) -> Result<Symbol, Error> {
        let nombre_ = nombre.len();
        log!(&env, "Longitud del nombre: {}", nombre_);
        
        // verificación de la firma de la función
        if nombre.len() == 0 { // nombre vacio, sale con error
          
            return Err(Error::NombreVacio);
        }
        if nombre.len() > 32 {// nombre muy largo, sale con error. No puede ir en cualquier storage
            return Err(Error::NombreMuyLargo);
        }
        //

        //implementación del contador
        // indicador del contador
        let key_contador = DataKey::ContadorSaludos;
        // lectura de la variable, 0 si no tiene valor
        let contador: u32 = env.storage().instance().get(&key_contador).unwrap_or(0);
        // incremento y guardado
        env.storage().instance().set(&key_contador, &(contador + 1));
        // guarda último saludo en memoria persistente= fato crítico del usuario
        env.storage().persistent().set(&DataKey::UltimoSaludo(usuario.clone()), &nombre);

        // extensión de TTL. primero en persistente, luego en instance
        env.storage().persistent().extend_ttl(&DataKey::UltimoSaludo(usuario), 100, 100);
        env.storage().instance().extend_ttl(100, 100);

        Ok(Symbol::new(&env, "Hola")) // retornar saludo
    }

    //implementación de funciones de consulta
    // valor por defecto del contador=0, con unwrap(0)
    pub fn get_contador(env: Env) -> u32 {env.storage().instance().get(&DataKey::ContadorSaludos).unwrap_or(0)}
    // puede no existir el saludo. El String se guarda en storage
    pub fn get_ultimo_saludo(env: Env, usuario: Address) -> Option<String> {env.storage().persistent().get(&DataKey::UltimoSaludo(usuario))}

    // implemetación de función administrativa
    pub fn reset_contador(env: Env, caller: Address) -> Result<(), Error> {
        // obtener admin y verificar. ?: Si get()=> None, se convierte en Err(NoInicializado) y la función retorna inmediatamente.
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NoInicializado)?;

    // verificar permisos. solo el admin resetear. si el caller no es admin, lo saca con error
    if caller != admin {
        return Err(Error::NoAutorizado);
    }
    // si el caller es el admin, puede resetear en 0 el contadorSaludos
    env.storage().instance().set(&DataKey::ContadorSaludos, &0u32);

    Ok(()) // retorna el contadorSaludos=0
    }
}


#[cfg(test)]
mod test {
   use super::*;
    use soroban_sdk::Env;
    use soroban_sdk:: testutils::Address; // permite Address::generate

    #[test]
    fn test_initialize() {
        let env = Env::default();
   //  let contract_id = env.register_contract(None, HelloContract);
        let contract_id = env.register(HelloContract,()); // cambiado
        let client = HelloContractClient::new(&env, &contract_id);
        
        //let admin = Address::generate(&env); // cambiada
        
        let admin = <soroban_sdk::Address as Address>::generate(&env);

        // Primera inicialización debe funcionar
        client.initialize(&admin);
        
        // Verificar contador en 0
        assert_eq!(client.get_contador(), 0);
    }
    // no inicializar dos veces
    #[test]
    #[should_panic(expected = "NoInicializado")]
    fn test_no_reinicializar() {
        let env = Env::default();
        let contract_id = env.register(HelloContract,());
        let client = HelloContractClient::new(&env, &contract_id);
        
        let admin = <soroban_sdk::Address as Address>::generate(&env);

      
        client.initialize(&admin);
        client.initialize(&admin);  // Segunda vez debe fallar
    }

    // hello exitoso con validaciones
    #[test]
    fn test_hello_exitoso() {
        let env = Env::default();
        let contract_id = env.register(HelloContract,());
        let client = HelloContractClient::new(&env, &contract_id);

        //  let admin = Address::generate(&env);
        let admin = <soroban_sdk::Address as Address>::generate(&env);
        let usuario = <soroban_sdk::Address as Address>::generate(&env);
      

//let usuario = Address::generate(&env);
        client.initialize(&admin);
        
        // ⭐ Usar String::from_str en lugar de Symbol::new
        let nombre = String::from_str(&env, "Ana");
        let resultado = client.hello(&usuario, &nombre);
        
        assert_eq!(resultado, Symbol::new(&env, "Hola"));
        assert_eq!(client.get_contador(), 1);
        assert_eq!(client.get_ultimo_saludo(&usuario), Some(nombre));
    }

    // nombreVacio falla
    #[test]
    #[should_panic(expected = "NombreVacio")]
    fn test_nombre_vacio() {
        let env = Env::default();
        let contract_id = env.register(HelloContract,());
        let client = HelloContractClient::new(&env, &contract_id);
        
        let admin = <soroban_sdk::Address as Address>::generate(&env);
        let usuario = <soroban_sdk::Address as Address>::generate(&env);

        client.initialize(&admin);
        
        // ⭐ Usar String::from_str para string vacío
        let vacio = String::from_str(&env, "");
        client.hello(&usuario, &vacio);  // Debe fallar
    }
    // reset solo admin
    #[test]
    fn test_reset_solo_admin() {
        let env = Env::default();
        let contract_id = env.register(HelloContract,());
        let client = HelloContractClient::new(&env, &contract_id);
        

        let otro =<soroban_sdk::Address as Address>::generate(&env);
        let admin = <soroban_sdk::Address as Address>::generate(&env);
        let usuario = <soroban_sdk::Address as Address>::generate(&env);

        client.initialize(&admin);
        
        // ⭐ Hacer saludos con String
        client.hello(&usuario, &String::from_str(&env, "Test"));
        assert_eq!(client.get_contador(), 1);
        
        // Admin puede resetear
        client.reset_contador(&admin);
        assert_eq!(client.get_contador(), 0);
    }
    // usuario no admin no resetea
    #[test]
    #[should_panic(expected = "NoAutorizado")]
    fn test_reset_no_autorizado() {
        let env = Env::default();
        let contract_id = env.register(HelloContract,());
        let client = HelloContractClient::new(&env, &contract_id);
        
       // let admin = Address::generate(&env);
       let otro =<soroban_sdk::Address as Address>::generate(&env);
       let admin = <soroban_sdk::Address as Address>::generate(&env);

        client.initialize(&admin);
        
        // Otro usuario intenta resetear
        client.reset_contador(&otro);  // Debe fallar
    }

}
