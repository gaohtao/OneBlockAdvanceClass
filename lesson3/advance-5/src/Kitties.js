import React, { useEffect, useState } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { useSubstrateState } from './substrate-lib'

import { TxButton } from './substrate-lib/components'
import KittyCards from './KittyCards'


//十六进制字符串转换为字节数组, 跳过0x起始字符
function HexString2Bytes(str) {
  str = str.substring(2);
  var pos = 0;
  var len = str.length;
  if (len % 2 !== 0) {
    return null;
  }
  len /= 2;
  var arrBytes = new Array();
  for (var i = 0; i < len; i++) {
    var s = str.substr(pos, 2);
    var v = parseInt(s, 16);
    arrBytes.push(v);
    pos += 2;
  }
  return arrBytes;
}


export default function Main (props) {  
  const { api, keyring, currentAccount } = useSubstrateState();
  const [kittyIndexes, setKittyIndexes] = useState([]);
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  useEffect(() => {
    const fetchKittyIndexes = async () => {
      const kittyIndex =  (await api.query.kittiesModule.nextKittyId()).toNumber();
      setKittyIndexes(Array.from(Array(kittyIndex).keys()));     
    };

    fetchKittyIndexes();
  }, [api, status, keyring, setKittyIndexes]);

  useEffect(() => {
    let unsub = null;

    const fetchKitties = async () => {
      

      // let kitty =  await api.query.kittiesModule.kitties(0);
      // console.log("3333 kitty=",kitty.toJSON())

      const kittyDNAs = await api.query.kittiesModule.kitties.multi(
        kittyIndexes
      );
      // console.log("3333 kittyDNAs[0]=",kittyDNAs[0].toJSON())
      // console.log("332211  kittyDNAs[0].dna=",kittyDNAs[0].toJSON().dna)
      
      // let bbb = HexString2Bytes(kittyDNAs[0].toJSON().dna);
      // console.log("bbb=", bbb);

      const kitties = kittyIndexes.map(kittyIndex => ({
        id: kittyIndex,
        dna:  HexString2Bytes(kittyDNAs[kittyIndex].toJSON().dna),
        owner: kittyDNAs[kittyIndex].toJSON().owner,
      }));
      console.log("4444  kitties=",kitties)

      setKitties(kitties);
    };

    fetchKitties();

    return () => {
      unsub && unsub();
    };
  }, [api, keyring, kittyIndexes, setKitties]);

  // console.log("### currentAccount=",currentAccount)
  // console.log("### kittyIndexes=",kittyIndexes)
  // console.log("### kitties=",kitties)

  return <Grid.Column width={16}>
    <h1>迷恋猫</h1>    
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <KittyCards
          kitties={kitties}
          accountPair={currentAccount}
          setStatus={setStatus}
        />
        <TxButton
          label="创建小猫" 
          type='SIGNED-TX'
          accountPair={currentAccount}
          setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'createKitty',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>

}
