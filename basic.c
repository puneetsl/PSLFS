#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "ds.h"
#include "partition.h"
#include "fstest1.h"
int main()
{
    FILE *f,*fp;
    char name[32],*command,p[10]="files",path[10],path2[10],path3[10],path4[10];
    ffolder root;
    freesec k;
    char i=0;
    strcpy(path,p);
    strcat(path,".psl");
    
    strcpy(path2,p);
    strcat(path2,".fol");
    strcpy(path3,p);
    strcat(path3,".fil");
    strcpy(path4,p);
    strcat(path4,".fs");
    
    
    f=fopen(path,"wb");
    strcpy(root.name,"pslos");
    fwrite(&i,sizeof(i),1,f);
    root.sector=1;
    root.upsector=0;
    root.insector=0;
    root.filesector=0;
    root.next=0;
    root.prev=0;
    root.size=0;
    root.properties[0]='r';
    root.properties[1]='w';
    root.properties[2]='v';
    fwrite(&root,sizeof(root),1,f);
    fclose(f);
    f=fopen(path,"r");
    fp=fopen(path2,"wb");
    k.next=0;
    fseek(f,0,2);
    
    
    k.sector=ftell(f);
    
    
    fwrite(&k,sizeof(k),1,fp);
    fclose(f);
    fclose(fp);
    fp=fopen(path3,"wb");
    fwrite(&k,sizeof(k),1,fp);
    fclose(fp);
    fp=fopen(path4,"wb");
    fwrite(&k,sizeof(k),1,fp);
    fclose(fp);
    makeUser();
    return 0;
    
}
